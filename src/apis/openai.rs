/// This file is not only for `OpenAI`. This is used for the Groq API as well, because Groq has openAI compatibility.
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIChoice {
    index: i32,
    message: OpenAIMessage,
    logprobs: Option<serde_json::Value>,
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIUsage {
    #[serde(rename = "prompt_tokens")]
    prompt: i32,
    #[serde(rename = "completion_tokens")]
    completion: i32,
    #[serde(rename = "total_tokens")]
    total: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIChatCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    system_fingerprint: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

impl OpenAIChatCompletion {
    pub fn get_text(&self) -> String {
        self.choices
            .iter()
            .map(|choice| choice.message.content.clone())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
}

pub async fn generate_content(
    base_url: &str,
    model: &str,
    api_key: &str,
    http_client: reqwest::Client,
    prompt: &str,
) -> Result<OpenAIChatCompletion, CommandError> {
    let response = http_client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&OpenAIRequest {
            model: model.to_string(),
            messages: vec![OpenAIMessage { role: "user".to_string(), content: prompt.to_string() }],
            temperature: 0.5,
        })
        .send()
        .await?
        .server_error()?
        .error_for_status()?
        .json::<OpenAIChatCompletion>()
        .await?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq() {
        let base_url = "https://api.groq.com/openai/v1";
        let model = "llama3-70b-8192";
        let api_key = dotenvy::var("GROQ_API_KEY").unwrap();
        let http_client = reqwest::Client::new();
        let prompt = "What is the last digit of pi?";

        let result = generate_content(base_url, model, &api_key, http_client, prompt).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.choices.len(), 1);
        println!("{response:?}");
    }
}
