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
}

pub async fn generate_content(
    base_url: &str,
    model: &str,
    api_key: &str,
    http_client: reqwest::Client,
    prompt: &str,
) -> Result<Result<ChatCompletion, Error>, CommandError> {
    let response = http_client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&Request {
            model: model.to_string(),
            messages: vec![Message { role: "user".to_string(), content: prompt.to_string() }],
            temperature: 0.5,
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
