use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const RETRY_COUNT: usize = 3;

#[derive(Serialize)]
struct Payload {
    pub prompt: String,
}

#[derive(Deserialize)]
struct Response {
    pub images: Vec<String>,
}

pub struct GeneratedResult {
    pub images: Vec<Vec<u8>>,
    pub duration: Duration,
}

pub async fn generate<S: Into<String>>(
    http_client: reqwest::Client,
    prompt: S,
) -> reqwest::Result<GeneratedResult> {
    let body = Payload {
        prompt: prompt.into(),
    };
    let mut retry = 0;
    let (response, duration) = loop {
        retry += 1;
        let start = Instant::now();
        match http_client
            .post("https://backend.craiyon.com/generate")
            .json(&body)
            .send()
            .await?
            .error_for_status()
        {
            Ok(response) => {
                break {
                    let duration = start.elapsed();
                    (response.json::<Response>().await?, duration)
                }
            }
            Err(err) => {
                let status = err.status();
                if let Some(status) = status {
                    log::warn!("HTTP error: {status}");
                };
                if retry < RETRY_COUNT {
                    let duration = if status == Some(StatusCode::TOO_MANY_REQUESTS) {
                        10
                    } else {
                        2
                    };
                    tokio::time::sleep(Duration::from_secs((retry * duration) as _)).await;
                    log::info!("Retrying ({retry})…");
                    continue;
                }
                log::warn!("Failed after {retry} retries");
                return Err(err);
            }
        };
    };

    let images = response
        .images
        .into_iter()
        .map(|data| base64::decode(data.replace('\n', "")).unwrap())
        .collect();

    Ok(GeneratedResult { images, duration })
}
