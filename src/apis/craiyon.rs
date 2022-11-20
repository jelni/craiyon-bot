use std::time::{Duration, Instant};

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

pub struct GenerationResult {
    pub images: Vec<Vec<u8>>,
    pub duration: Duration,
}

pub async fn generate<S: Into<String>>(
    http_client: reqwest::Client,
    prompt: S,
) -> reqwest::Result<GenerationResult> {
    let body = Payload { prompt: prompt.into() };
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
                log::warn!("{err}");
                if retry < RETRY_COUNT {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                return Err(err);
            }
        };
    };

    let images = response
        .images
        .into_iter()
        .map(|data| base64::decode(data.replace('\n', "")).unwrap())
        .collect();

    Ok(GenerationResult { images, duration })
}
