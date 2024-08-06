use std::env;

use log::info;
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

#[derive(Serialize)]
pub struct Request {
    pub model_name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodel_name: Option<&'static str>,
    pub prompt: String,
    pub negative_prompt: String,
    pub image_size: ImageSize,
    pub num_inference_steps: u8,
    pub expand_prompt: bool,
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
pub struct Response {
    pub images: Vec<Image>,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn generate(
    http_client: &reqwest::Client,
    request: Request,
) -> Result<Response, CommandError> {
    let response = http_client
        .post(format!("https://fal.run/fal-ai/{}", request.model_name))
        .header(AUTHORIZATION, format!("Key {}", env::var("FAL_API_KEY").unwrap()))
        .json(&request)
        .send()
        .await?
        .server_error()?;

    info!("response: {response:?}");
    info!("status: {}", response.status());
    if response.status() == StatusCode::OK {
        Ok(response.json::<Response>().await?)
    } else {
        Err(response.json::<ErrorResponse>().await?.error.into())
    }
}
