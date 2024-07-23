use std::env;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

#[derive(Serialize)]
pub struct FalRequest {
    pub model_name: &'static str,
    pub prompt: String,
    pub negative_prompt: String,
    pub image_size: ImageSize,
    pub num_inference_steps: u8,
    pub guidance_scale: u8,
    pub num_images: u8,
    pub enable_safety_checker: bool,
    pub format: &'static str,
}

#[derive(Serialize)]
pub struct ImageSize {
    pub height: u16,
    pub width: u16,
}

#[derive(Deserialize)]
pub struct FalResponse {
    pub images: Vec<Image>,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
    pub content_type: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    pub message: String,
}

pub async fn generate(
    http_client: &reqwest::Client,
    request: FalRequest,
) -> Result<FalResponse, CommandError> {
    let response = http_client
        .post(format!("https://fal.run/fal-ai/{}", request.model_name))
        .header(AUTHORIZATION, format!("Key {}", env::var("FAL_API_KEY").unwrap()))
        .json(&request)
        .send()
        .await?
        .server_error()?;

    if response.status() == StatusCode::OK {
        Ok(response.json::<FalResponse>().await?)
    } else {
        Err(response.json::<ErrorResponse>().await?.error.message.into())
    }
}
