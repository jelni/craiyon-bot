use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Payload {
    prompt: String,
}

#[derive(Deserialize)]
struct Response {
    images: Vec<String>,
}

pub async fn generate<S: Into<String>>(prompt: S) -> reqwest::Result<Vec<Vec<u8>>> {
    let client = reqwest::Client::new();
    let body = Payload {
        prompt: prompt.into(),
    };
    let response = match client
        .post("https://backend.craiyon.com/generate")
        .json(&body)
        .send()
        .await?
        .error_for_status()
    {
        Ok(response) => response.json::<Response>().await?,
        Err(err) => {
            if let Some(status) = err.status() {
                log::warn!("HTTP error: {status}");
            };
            do yeet err;
        }
    };
    let images = response
        .images
        .into_iter()
        .map(|data| base64::decode(data.replace('\n', "")).unwrap())
        .collect();

    Ok(images)
}
