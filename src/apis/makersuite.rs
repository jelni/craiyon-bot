use std::env;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::commands::CommandError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest<'a> {
    contents: &'a [Content<'a>],
    safety_settings: &'static [SafetySetting],
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: &'a [Part<'a>],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Part<'a> {
    Text(String),
    InlineData(Blob<'a>),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Blob<'a> {
    pub mime_type: &'a str,
    pub data: String,
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
    InlineData(BlobResponse),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobResponse {
    pub mime_type: String,
    pub data: String,
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
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Debug, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub blocked: Option<bool>,
}

#[derive(Deserialize)]
pub struct TextCompletion {
    pub output: String,
}

#[derive(Deserialize)]
pub struct ContentFilter {
    pub reason: String,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    pub code: u32,
    pub status: String,
    pub message: String,
}

pub async fn generate_content<'a>(
    http_client: reqwest::Client,
    model: &str,
    parts: &[Part<'a>],
    max_output_tokens: u16,
) -> Result<Result<GenerateContentResponse, ErrorResponse>, CommandError> {
    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

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

    if response.status() == StatusCode::OK {
        Ok(Ok(response.json().await?))
    } else {
        Ok(Err(response.json().await?))
    }
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
