use std::borrow::Cow;
use std::env;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::commands::CommandError;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest<'a> {
    contents: &'a [Content<'a>],
    safety_settings: &'static [SafetySetting],
    generation_config: GenerationConfig,
}

#[derive(Serialize, Debug)]
struct Content<'a> {
    parts: &'a [Part],
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
    InlineData(Blob),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub mime_type: Cow<'static, str>,
    pub data: String,
}

#[derive(Serialize, Debug)]
pub struct SafetySetting {
    pub category: &'static str,
    pub threshold: &'static str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    max_output_tokens: u16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    pub prompt_feedback: Option<PromptFeedback>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Option<ContentResponse>,
    pub finish_reason: String,
    pub citation_metadata: Option<CitationMetadata>,
}

#[derive(Deserialize, Debug)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PartResponse {
    Text(String),
    InlineData(BlobResponse),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlobResponse {
    pub mime_type: String,
    pub data: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Deserialize, Debug)]
pub struct CitationSource {
    pub uri: Option<String>,
    pub license: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    pub block_reason: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Debug, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    #[serde(default)]
    pub blocked: bool,
}

#[derive(Deserialize, Debug)]
pub struct TextCompletion {
    pub output: String,
}

#[derive(Deserialize, Debug)]
pub struct ContentFilter {
    pub reason: String,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize, Debug)]
pub struct Error {
    pub code: u32,
    pub status: String,
    pub message: String,
}

pub async fn generate_content(
    http_client: reqwest::Client,
    model: &str,
    parts: &[Part],
    max_output_tokens: u16,
) -> Result<Result<GenerateContentResponse, ErrorResponse>, CommandError> {
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
            generation_config: GenerationConfig { max_output_tokens },
        })
        .send()
        .await?;

    // This is a stream, save all of the responses to a vec.
    let vec: Vec<GenerateContentResponse> = response.json().await?;
    // Combine all of the responses into one.
    let response = vec.into_iter().fold(
        GenerateContentResponse { candidates: Vec::new(), prompt_feedback: None },
        |mut acc, mut x| {
            acc.candidates.append(&mut x.candidates);
            acc.prompt_feedback = x.prompt_feedback.or(acc.prompt_feedback);
            acc
        },
    );

    Ok(Ok(response))
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
) -> Result<Result<GenerateTextResponse, ErrorResponse>, CommandError> {
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
        Ok(Err(response.json().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use std::env;

    #[tokio::test]
    async fn test_generate_content() {
        let http_client = Client::new();
        let model = "gemini-pro";
        let parts =
            vec![Part::Text("Write a long essay about the history of bananas.".to_string())];
        let max_output_tokens = 512;

        // Set the API key in the environment for the test
        dotenvy::dotenv().ok();
        env::set_var("MAKERSUITE_API_KEY", std::env::var("MAKERSUITE_API_KEY").unwrap());

        let result = generate_content(http_client, model, &parts, max_output_tokens).await;

        println!("{result:?}");
        assert!(result.is_ok());
        // Check if the content is none
        let content = result.unwrap().unwrap();
        let content = content.candidates.first().unwrap().content.as_ref();
        assert!(content.is_some(), "content is none");
    }
}
