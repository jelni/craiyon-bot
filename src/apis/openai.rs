use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    // pub finish_reason: String,
}

#[derive(Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    pub message: String,
    pub r#type: String,
    pub param: String,
    pub code: String,
}

#[derive(Deserialize, Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
}

pub async fn chat_completion(
    http_client: reqwest::Client,
    api_key: &str,
    base_url: &str,
    model: &str,
    prompt: &str,
    temperature: f32,
) -> Result<Result<ChatCompletion, Error>, CommandError> {
    let response = http_client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&Request {
            model: model.into(),
            messages: vec![Message { role: "user".into(), content: prompt.into() }],
            temperature,
            max_tokens: 256,
        })
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
