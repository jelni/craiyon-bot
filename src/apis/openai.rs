use core::fmt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Serialize)]
struct Request<'a> {
    messages: &'a [Message<'a>],
    model: &'static str,
    max_tokens: u16,
}

#[derive(Serialize)]
pub struct Message<'a> {
    pub role: &'static str,
    pub content: Cow<'a, str>,
}

#[derive(Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: MessageResponse,
    pub finish_reason: String,
}

#[derive(Deserialize)]
pub struct MessageResponse {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ErrorCode {
    String(String),
    U32(u32),
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::String(code) => f.write_str(&code),
            ErrorCode::U32(code) => write!(f, "{code}"),
        }
    }
}

pub async fn chat_completion(
    http_client: reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &'static str,
    max_tokens: u16,
    messages: &[Message<'_>],
) -> Result<Result<ChatCompletion, Error>, CommandError> {
    let response = http_client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .json(&Request { messages, model, max_tokens })
        .send()
        .await?
        .server_error()?;

    if response.status() == StatusCode::OK {
        let response = response.json::<ChatCompletion>().await?;
        Ok(Ok(response))
    } else {
        let response = response.json::<ErrorResponse>().await?;
        Ok(Err(response.error))
    }
}
