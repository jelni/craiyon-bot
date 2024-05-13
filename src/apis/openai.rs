use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    index: i32,
    message: Message,
    logprobs: Option<serde_json::Value>,
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Usage {
    #[serde(rename = "prompt_tokens")]
    prompt: i32,
    #[serde(rename = "completion_tokens")]
    completion: i32,
    #[serde(rename = "total_tokens")]
    total: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    system_fingerprint: String,
    choices: Vec<Choice>,
    usage: Usage,
}

impl ChatCompletion {
    pub fn get_text(&self) -> String {
        self.choices
            .iter()
            .map(|choice| choice.message.content.clone())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
) -> Result<ChatCompletion, CommandError> {
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
        .server_error()?
        .error_for_status()?
        .json::<ChatCompletion>()
        .await?;

    Ok(response)
}
