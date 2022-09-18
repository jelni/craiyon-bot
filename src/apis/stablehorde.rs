use std::env;
use std::time::Instant;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use super::craiyon::GenerationResult;

#[derive(Serialize)]
struct Payload {
    prompt: String,
    params: Params,
    api_key: String,
}

#[derive(Serialize)]
struct Params {
    n: usize,
    width: usize,
    height: usize,
    cfg_scale: f32,
    steps: usize,
}

#[derive(Deserialize)]
struct Image {
    img: String,
}

pub async fn generate<S: Into<String>>(
    http_client: reqwest::Client,
    prompt: S,
) -> reqwest::Result<Result<GenerationResult, String>> {
    let start = Instant::now();
    let response = http_client
        .post("https://stablehorde.net/api/latest/generate/sync")
        .json(&Payload {
            prompt: prompt.into(),
            params: Params { n: 4, width: 512, height: 512, cfg_scale: 7.5, steps: 100 },
            api_key: env::var("STABLEHORDE_TOKEN").unwrap(),
        })
        .send()
        .await?;
    let duration = start.elapsed();

    match response.status() {
        StatusCode::OK => {
            let images = response
                .json::<Vec<Image>>()
                .await?
                .into_iter()
                .map(|i| base64::decode(i.img).unwrap())
                .collect::<Vec<_>>();
            Ok(Ok(GenerationResult { images, duration }))
        }
        status => {
            let error =
                response.json::<String>().await.unwrap_or_else(|_| "zjebalo sie".to_string());
            Ok(Err(format!("{}: {error}", status.as_u16())))
        }
    }
}
