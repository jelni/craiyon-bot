use reqwest::{StatusCode, Url};
use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub title: Option<String>,
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
) -> Result<Result<Data, Error>, CommandError> {
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
                    ("timeout", "1m"),
                    ("viewport.width", "1280"),
                    ("viewport.height", "640"),
                    ("wait_for_timeout", "5s"),
                    ("wait_until", "load"),
                ],
            )
            .unwrap(),
        )
        .send()
        .await?
        .server_error()?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<Response>().await?.data)),
        _ => Ok(Err(response.json::<Error>().await?)),
    }
}
