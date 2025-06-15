use std::borrow::Cow;
use std::time::Duration;
use std::{env, fmt};

use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::sync::mpsc;
use url::Url;

use crate::commands::CommandError;

pub enum GenerationError {
    Network(reqwest::Error),
    Google(Vec<Error>),
}

#[derive(Deserialize)]
pub struct FileResponse {
    file: File,
}

#[derive(Deserialize)]
pub struct File {
    name: String,
    pub uri: String,
    state: State,
    error: Option<Status>,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum State {
    Processing,
    Active,
    Failed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    code: u32,
    message: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

pub async fn upload_file(
    http_client: &reqwest::Client,
    file: tokio::fs::File,
    size: u64,
    mime_type: &str,
) -> Result<File, CommandError> {
    // what the hell
    let response = http_client
        .post(
            Url::parse_with_params(
                "https://generativelanguage.googleapis.com/upload/v1beta/files",
                [("key", env::var("MAKERSUITE_API_KEY").unwrap())],
            )
            .unwrap(),
        )
        .header("X-Goog-Upload-Protocol", "resumable")
        .header("X-Goog-Upload-Command", "start")
        .header("X-Goog-Upload-Header-Content-Length", &size.to_string())
        .header("X-Goog-Upload-Header-Content-Type", mime_type)
        .json(&Value::Object(Map::new()))
        .send()
        .await?;

    let upload_url = response.headers()["X-Goog-Upload-URL"].to_str().unwrap();

    let mut file = http_client
        .post(upload_url)
        .header("X-Goog-Upload-Offset", "0")
        .header("X-Goog-Upload-Command", "upload, finalize")
        .body(file)
        .send()
        .await?
        .json::<FileResponse>()
        .await?
        .file;

    while matches!(file.state, State::Processing) {
        tokio::time::sleep(Duration::from_secs(1)).await;

        file = http_client
            .get(
                Url::parse_with_params(
                    &format!("https://generativelanguage.googleapis.com/v1beta/{}", file.name),
                    [("key", env::var("MAKERSUITE_API_KEY").unwrap())],
                )
                .unwrap(),
            )
            .send()
            .await?
            .json::<File>()
            .await?;
    }

    if let Some(error) = file.error {
        return Err(format!("Google error {}: {}", error.code, error.message).into());
    }

    Ok(file)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest<'a> {
    contents: Cow<'a, [Content<'a>]>,
    safety_settings: &'static [SafetySetting],
    system_instruction: Option<Content<'a>>,
    generation_config: GenerationConfig,
}

#[derive(Clone, Serialize)]
pub struct Content<'a> {
    pub parts: Cow<'a, [Part<'a>]>,
    pub role: Option<&'static str>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Part<'a> {
    Text(Cow<'a, str>),
    FileData(FileData),
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    pub file_uri: String,
}

#[derive(Serialize)]
pub struct SafetySetting {
    pub category: &'static str,
    pub threshold: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    max_output_tokens: u16,
    thinking_config: ThinkingConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThinkingConfig {
    thinking_budget: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    pub prompt_feedback: Option<PromptFeedback>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Option<ContentResponse>,
    pub finish_reason: Option<String>,
    pub citation_metadata: Option<CitationMetadata>,
}

#[derive(Deserialize)]
pub struct ContentResponse {
    pub parts: Option<Vec<PartResponse>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PartResponse {
    Text(String),
    InlineData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Deserialize)]
pub struct CitationSource {
    pub uri: Option<String>,
    pub license: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    pub block_reason: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Deserialize)]
pub struct SafetyRating {
    pub category: String,
    #[serde(default)]
    pub blocked: bool,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Google error {}: {}", self.code, self.message)
    }
}

pub async fn stream_generate_content<'a>(
    http_client: reqwest::Client,
    tx: mpsc::UnboundedSender<Result<GenerateContentResponse, GenerationError>>,
    model: &str,
    contents: Cow<'a, [Content<'a>]>,
    system_instruction: Option<Content<'a>>,
    max_output_tokens: u16,
) {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent"
    );

    let response = http_client
        .post(
            Url::parse_with_params(&url, [("key", env::var("MAKERSUITE_API_KEY").unwrap())])
                .unwrap(),
        )
        .json(&GenerateContentRequest {
            contents,
            safety_settings: &[
                SafetySetting { category: "HARM_CATEGORY_HARASSMENT", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_HATE_SPEECH", threshold: "BLOCK_NONE" },
                SafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    threshold: "BLOCK_NONE",
                },
                SafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT",
                    threshold: "BLOCK_NONE",
                },
                SafetySetting {
                    category: "HARM_CATEGORY_CIVIC_INTEGRITY",
                    threshold: "BLOCK_NONE",
                },
            ],
            system_instruction,
            generation_config: GenerationConfig {
                max_output_tokens,
                thinking_config: ThinkingConfig { thinking_budget: 0 },
            },
        })
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(err) => {
            tx.send(Err(GenerationError::Network(err))).unwrap();
            return;
        }
    };

    let status = response.status();

    if status != StatusCode::OK {
        let error_response = response.json::<Vec<ErrorResponse>>().await;

        match error_response {
            Ok(error_response) => {
                tx.send(Err(GenerationError::Google(
                    error_response.into_iter().map(|error| error.error).collect(),
                )))
                .unwrap();
            }
            Err(err) => tx.send(Err(GenerationError::Network(err))).unwrap(),
        }

        return;
    }

    let mut buffer = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(part) = stream.next().await {
        let part = match part {
            Ok(part) => part,
            Err(err) => {
                tx.send(Err(GenerationError::Network(err))).unwrap();
                return;
            }
        };

        buffer.extend(&part);

        if let Some(stripped) = buffer.strip_prefix(b"[") {
            buffer = stripped.into();
        }

        if let Some(stripped) = buffer.strip_suffix(b"\n]") {
            buffer = stripped.into();
        }

        while let Some(index) = buffer.windows(4).position(|window| window == b"\n,\r\n") {
            let (first, rest) = buffer.split_at(index);
            tx.send(Ok(serde_json::from_str(&String::from_utf8(first.into()).unwrap()).unwrap()))
                .unwrap();
            buffer = rest[4..].into();
        }
    }

    tx.send(Ok(serde_json::from_str(&String::from_utf8(buffer).unwrap()).unwrap())).unwrap();
}
