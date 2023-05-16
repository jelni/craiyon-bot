use std::env;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::commands::CommandError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    prompt: TextPrompt<'a>,
    max_output_tokens: u32,
    safety_settings: &'a [SafetySetting],
}

#[derive(Serialize)]
struct TextPrompt<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct SafetySetting {
    category: &'static str,
    threshold: &'static str,
}

#[derive(Deserialize)]
pub struct Response {
    pub candidates: Option<Vec<TextCompletion>>,
    pub filters: Option<Vec<ContentFilter>>,
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
pub struct SafetyFeedback {
    pub rating: String,
    pub setting: String,
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

pub async fn generate_text(
    http_client: reqwest::Client,
    prompt: &str,
    max_output_tokens: u32,
) -> Result<Result<Response, ErrorResponse>, CommandError> {
    let response = http_client
        .post(
            Url::parse_with_params(
                concat!(
                    "https://generativelanguage.googleapis.com",
                    "/v1beta2/models/text-bison-001:generateText"
                ),
                [("key", env::var("MAKERSUITE_API_KEY").unwrap())],
            )
            .unwrap(),
        )
        .json(&Payload {
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

    if let StatusCode::OK = response.status() {
        Ok(Ok(response.json().await?))
    } else {
        Ok(Err(response.json().await?))
    }
}
