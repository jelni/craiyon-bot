use std::env;

use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Serialize)]
struct Payload<'a> {
    prompt: &'a str,
    format: &'static str,
    enable_safety_checker: bool,
}

#[derive(Deserialize)]
pub struct Response {
    pub images: Vec<Image>,
    pub timings: Timings,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
pub struct Timings {
    pub inference: f64,
}

#[derive(Deserialize)]
struct ErrorResponse {
    detail: String,
}

pub async fn generate(
    http_client: reqwest::Client,
    model: &str,
    prompt: &str,
) -> Result<Response, CommandError> {
    let response = http_client
        .post(format!("https://fal.run/{model}"))
        .header(AUTHORIZATION, format!("Key {}", env::var("FAL_API_KEY").unwrap()))
        .json(&Payload { prompt, enable_safety_checker: false, format: "png" })
        .send()
        .await?
        .server_error()?;

    if response.status() == StatusCode::OK {
        Ok(response.json().await?)
    } else {
        Err(response.json::<ErrorResponse>().await?.detail.into())
    }
}
