use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct TogetherImageRequest {
    pub model: String,
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub n: u32,
    pub response_format: String,
}

pub struct TogetherClient {
    client: Client,
}

#[derive(Deserialize, Debug)]
pub struct TogetherImageResponse {
    pub data: Vec<TogetherImageData>,
}

#[derive(Deserialize, Debug)]
pub struct TogetherImageData {
    pub b64_json: String,
}

impl TogetherClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn generate_image(
        &self,
        req: TogetherImageRequest,
    ) -> Result<TogetherImageResponse, String> {
        let api_key = env::var("TOGETHER_API_KEY").map_err(|_| "TOGETHER_API_KEY missing".to_string())?;
        let response = self
            .client
            .post("https://api.together.xyz/v1/images/generations")
            .bearer_auth(api_key)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Together API error: HTTP {}", response.status()));
        }

        let response = response.json::<TogetherImageResponse>().await.map_err(|e| e.to_string())?;
        Ok(response)
    }
}
