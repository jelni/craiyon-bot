use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

static CRAIYON_VERSION: &str = "35s5hfwn9n78gb06";

#[derive(Serialize)]
struct Payload<'a> {
    prompt: &'a str,
    version: &'static str,
}

#[derive(Deserialize)]
struct Response {
    images: Vec<String>,
}

pub struct GenerationResult {
    pub images: Vec<String>,
    pub duration: Duration,
}

pub async fn draw(
    http_client: reqwest::Client,
    prompt: &str,
) -> Result<GenerationResult, CommandError> {
    let start = Instant::now();
    let response = http_client
        .post("https://api.craiyon.com/draw")
        .json(&Payload { prompt, version: CRAIYON_VERSION })
        .send()
        .await?
        .server_error()?
        .error_for_status()?
        .json::<Response>()
        .await?;

    let duration = start.elapsed();

    let images =
        response.images.into_iter().map(|path| format!("https://img.craiyon.com/{path}")).collect();

    Ok(GenerationResult { images, duration })
}
