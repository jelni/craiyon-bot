use std::env;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

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
struct RequestId {
    id: String,
}

#[derive(Deserialize)]
pub struct Status {
    pub done: bool,
    pub waiting: usize,
    pub processing: usize,
    pub finished: usize,
    pub queue_position: usize,
    pub wait_time: usize,
    pub generations: Option<Vec<Generation>>,
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.waiting == other.waiting
            && self.processing == other.processing
            && self.finished == other.finished
            && self.queue_position == other.queue_position
            && self.wait_time == other.wait_time
    }
}

#[derive(Deserialize)]
pub struct Generation {
    pub img: String,
    pub server_name: String,
}

pub async fn generate<S: Into<String>>(
    http_client: reqwest::Client,
    prompt: S,
) -> reqwest::Result<Result<String, String>> {
    let response = http_client
        .post("https://stablehorde.net/api/v1/generate/async")
        .json(&Payload {
            prompt: prompt.into(),
            params: Params { n: 4, width: 512, height: 512, cfg_scale: 7.5, steps: 30 },
            api_key: env::var("STABLEHORDE_TOKEN").unwrap(),
        })
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<RequestId>().await?.id)),
        status => {
            let error =
                response.json::<String>().await.unwrap_or_else(|_| "zjebalo sie".to_string());
            Ok(Err(format!("{}: {error}", status.as_u16())))
        }
    }
}

async fn generation_info(
    http_client: reqwest::Client,
    action: &str,
    request_id: &str,
) -> reqwest::Result<Result<Status, String>> {
    let response = http_client
        .get(format!("https://stablehorde.net/api/v1/generate/{action}/{request_id}"))
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<Status>().await?)),
        status => {
            let error =
                response.json::<String>().await.unwrap_or_else(|_| "zjebalo sie".to_string());
            Ok(Err(format!("{}: {error}", status.as_u16())))
        }
    }
}

pub async fn status(
    http_client: reqwest::Client,
    request_id: &str,
) -> reqwest::Result<Result<Status, String>> {
    generation_info(http_client, "check", request_id).await
}

pub async fn results(
    http_client: reqwest::Client,
    request_id: &str,
) -> reqwest::Result<Result<Vec<Generation>, String>> {
    match generation_info(http_client, "prompt", request_id).await? {
        Ok(status) => Ok(Ok(status.generations.unwrap())),
        Err(err) => Ok(Err(err)),
    }
}
