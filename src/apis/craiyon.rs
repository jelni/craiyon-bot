use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

static DRAW_VERSION: &str = "35s5hfwn9n78gb06";
static SEARCH_VERSION: &str = "hpv3obayw36clkqp";

#[derive(Serialize)]
struct Payload<'a> {
    model: Model,
    prompt: &'a str,
    negative_prompt: &'a str,
    version: &'static str,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    Art,
    Drawing,
    Photo,
    None,
}

#[derive(Deserialize)]
struct Response {
    images: Vec<String>,
    next_prompt: String,
}

pub struct GenerationResult {
    pub images: Vec<String>,
    pub next_prompt: String,
    pub duration: Duration,
}

pub async fn draw(
    http_client: reqwest::Client,
    model: Model,
    negative_prompt: &str,
    prompt: &str,
) -> Result<GenerationResult, CommandError> {
    let start = Instant::now();
    let response = http_client
        .post("https://api.craiyon.com/v3")
        .json(&Payload { model, prompt, negative_prompt, version: DRAW_VERSION })
        .send()
        .await?
        .server_error()?
        .error_for_status()?
        .json::<Response>()
        .await?;

    let duration = start.elapsed();

    let images =
        response.images.into_iter().map(|path| format!("https://img.craiyon.com/{path}")).collect();

    Ok(GenerationResult { images, next_prompt: response.next_prompt, duration })
}

#[derive(Deserialize)]
pub struct SearchResult {
    pub image_id: String,
    pub prompt: String,
}

pub async fn search(
    http_client: reqwest::Client,
    text: &str,
) -> Result<Vec<SearchResult>, CommandError> {
    let results = http_client
        .post("https://search.craiyon.com/search")
        .form(&[("text", text), ("version", SEARCH_VERSION)])
        .send()
        .await?
        .server_error()?
        .error_for_status()?
        .json::<Vec<SearchResult>>()
        .await?;

    Ok(results)
}
