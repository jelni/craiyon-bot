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
    http_client: reqwest::Client,
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
        return Err(CommandError::Custom(format!(
            "Google error {}: {}",
            error.code, error.message
        )));
    }

    Ok(file)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest<'a> {
    contents: &'a [Content<'a>],
    safety_settings: &'static [SafetySetting],
    system_instruction: Option<Content<'a>>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: &'a [Part<'a>],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Part<'a> {
    Text(Cow<'a, str>),
    FileData(FileData),
}

#[derive(Serialize)]
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
    pub finish_reason: String,
    pub citation_metadata: Option<CitationMetadata>,
}

#[derive(Deserialize)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
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

#[derive(Deserialize)]
pub struct ContentFilter {
    pub reason: String,
    pub message: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Google error {}: {}", self.code, self.message)
    }
}

pub async fn stream_generate_content(
    http_client: reqwest::Client,
    tx: mpsc::UnboundedSender<Result<GenerateContentResponse, GenerationError>>,
    model: &str,
    parts: &[Part<'_>],
    system_instruction: Option<&[Part<'_>]>,
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
            contents: &[Content { parts }],
            safety_settings: &[
                SafetySetting { category: "HARM_CATEGORY_HATE_SPEECH", threshold: "BLOCK_NONE" },
                SafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    threshold: "BLOCK_NONE",
                },
                SafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT",
                    threshold: "BLOCK_NONE",
                },
                SafetySetting { category: "HARM_CATEGORY_HARASSMENT", threshold: "BLOCK_NONE" },
            ],
            system_instruction: system_instruction
                .map(|system_instruction| Content { parts: system_instruction }),
            generation_config: GenerationConfig { max_output_tokens },
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
            tx.send(Ok(serde_json::from_str(&String::from_utf8_lossy(first)).unwrap())).unwrap();
            buffer = rest[4..].into();
        }
    }

    tx.send(Ok(serde_json::from_str(&String::from_utf8_lossy(&buffer)).unwrap())).unwrap();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateTextRequest<'a> {
    prompt: TextPrompt<'a>,
    safety_settings: &'a [SafetySetting],
    max_output_tokens: u16,
}

#[derive(Serialize)]
pub struct TextPrompt<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
pub struct GenerateTextResponse {
    pub candidates: Option<Vec<TextCompletionResponse>>,
    pub filters: Option<Vec<ContentFilter>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextCompletionResponse {
    pub output: String,
    pub citation_metadata: Option<CitationMetadata>,
}

pub async fn generate_text(
    http_client: reqwest::Client,
    prompt: &str,
    max_output_tokens: u16,
) -> Result<Result<GenerateTextResponse, Error>, CommandError> {
    let response = http_client
        .post(
            Url::parse_with_params(
                concat!(
                    "https://generativelanguage.googleapis.com",
                    "/v1beta/models/text-bison-001:generateText"
                ),
                [("key", env::var("MAKERSUITE_API_KEY").unwrap())],
            )
            .unwrap(),
        )
        .json(&GenerateTextRequest {
            prompt: TextPrompt { text: prompt },
            max_output_tokens,
            safety_settings: &[
                SafetySetting { category: "HARM_CATEGORY_DEROGATORY", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_TOXICITY", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_VIOLENCE", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_SEXUAL", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_MEDICAL", threshold: "BLOCK_NONE" },
                SafetySetting { category: "HARM_CATEGORY_DANGEROUS", threshold: "BLOCK_NONE" },
            ],
        })
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        Ok(Ok(response.json().await?))
    } else {
        Ok(Err(response.json::<ErrorResponse>().await?.error))
    }
}
