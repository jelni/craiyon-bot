use reqwest::{StatusCode, Url};
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub title: String,
    pub screenshot: Screenshot,
}

#[derive(Deserialize)]
pub struct Screenshot {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
    pub more: String,
}

pub async fn screenshot(
    http_client: reqwest::Client,
    url: Url,
) -> reqwest::Result<Result<Data, Error>> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://api.microlink.io/",
                [
                    ("url", url.as_str()),
                    ("color_scheme", "dark"),
                    ("ping", "false"),
                    ("prerender", "true"),
                    ("screenshot", "true"),
                    ("timeout", "60s"),
                    ("viewport.width", "1280"),
                    ("viewport.height", "640"),
                    ("wait_until", "load"),
                ],
            )
            .unwrap(),
        )
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<Response>().await?.data)),
        _ => Ok(Err(response.json::<Error>().await?)),
    }
}
