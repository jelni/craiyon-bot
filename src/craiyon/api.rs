use std::time::{Duration, Instant};

use super::models::{GeneratedResult, Payload, Response};

const RETRY_COUNT: usize = 3;

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
                if let Some(status) = err.status() {
                    log::warn!("HTTP error: {status}");
                };
                if retry <= RETRY_COUNT {
                    tokio::time::sleep(Duration::from_secs(2)).await;
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
