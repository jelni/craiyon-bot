use std::env;
use std::time::Duration;

use reqwest::{RequestBuilder, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

trait StableHordeClientAgent {
    fn client_agent_header(self) -> Self;
}

impl StableHordeClientAgent for RequestBuilder {
    fn client_agent_header(self) -> Self {
        self.header("Client-Agent", env::var("STABLEHORDE_CLIENT").unwrap())
    }
}

#[derive(Serialize)]
struct GenerationInput<'a> {
    prompt: &'a str,
    models: Vec<&'static str>,
    params: Params,
    nsfw: bool,
    r2: bool,
    shared: bool,
}

#[derive(Serialize)]
struct Params {
    n: u32,
    width: u32,
    height: u32,
    sampler_name: &'static str,
    steps: u32,
    karras: bool,
}

#[derive(Deserialize)]
struct RequestId {
    id: String,
}

#[derive(Deserialize)]
struct RequestError {
    message: String,
}

#[derive(PartialEq, Deserialize)]
pub struct Status {
    pub done: bool,
    pub faulted: bool,
    pub is_possible: bool,
    // use u32 for these fields once Stable Horde fixes the race condition
    pub waiting: i8,
    pub processing: i8,
    pub finished: i8,
    pub queue_position: u32,
    pub wait_time: u32,
}

#[derive(Deserialize)]
pub struct Generations {
    pub generations: Vec<Generation>,
}

#[derive(Deserialize)]
pub struct Generation {
    pub img: String,
    pub worker_id: String,
    pub worker_name: String,
}

pub async fn generate(
    http_client: reqwest::Client,
    prompt: &str,
    model: &'static str,
    size: (u32, u32),
) -> Result<Result<String, String>, CommandError> {
    let response = http_client
        .post("https://stablehorde.net/api/v2/generate/async")
        .json(&GenerationInput {
            models: vec![model],
            prompt,
            params: Params {
                n: 4,
                width: size.0,
                height: size.1,
                sampler_name: "k_euler",
                steps: 24,
                karras: true,
            },
            nsfw: true,
            r2: true,
            shared: true,
        })
        .header("apikey", env::var("STABLEHORDE_TOKEN").unwrap())
        .client_agent_header()
        .send()
        .await?
        .server_error()?;

    match response.status() {
        StatusCode::ACCEPTED => Ok(Ok(response.json::<RequestId>().await?.id)),
        status => {
            let error = response
                .json::<RequestError>()
                .await
                .map_or_else(|_| "zjebalo sie".into(), |e| e.message);
            Ok(Err(format!("{}: {error}", status.as_u16())))
        }
    }
}

async fn generation_info<O: DeserializeOwned>(
    http_client: reqwest::Client,
    action: &str,
    request_id: &str,
) -> Result<Result<O, String>, CommandError> {
    let url = Url::parse(&format!("https://stablehorde.net/api/v2/generate/{action}/{request_id}"))
        .unwrap();
    let response = loop {
        match http_client.get(url.clone()).client_agent_header().send().await {
            Err(err) if err.is_request() => {
                log::warn!("{err}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            response => break response?.server_error(),
        }
    }?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<O>().await?)),
        status => {
            let error = response
                .json::<RequestError>()
                .await
                .map_or_else(|_| "zjebalo sie".into(), |e| e.message);
            Ok(Err(format!("{}: {error}", status.as_u16())))
        }
    }
}

pub async fn check(
    http_client: reqwest::Client,
    request_id: &str,
) -> Result<Result<Status, String>, CommandError> {
    generation_info::<Status>(http_client, "check", request_id).await
}

pub async fn results(
    http_client: reqwest::Client,
    request_id: &str,
) -> Result<Result<Vec<Generation>, String>, CommandError> {
    match generation_info::<Generations>(http_client, "status", request_id).await? {
        Ok(status) => Ok(Ok(status.generations)),
        Err(err) => Ok(Err(err)),
    }
}

pub async fn cancel_generation(
    http_client: reqwest::Client,
    request_id: &str,
) -> reqwest::Result<()> {
    http_client
        .delete(format!("https://stablehorde.net/api/v2/generate/status/{request_id}"))
        .send()
        .await?;

    Ok(())
}
