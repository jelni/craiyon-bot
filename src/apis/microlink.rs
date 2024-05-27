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
    let mut params = vec![
        ("url", url.as_str()),
        ("adblock", "false"),
        ("color_scheme", "dark"),
        ("ping", "false"),
        ("prerender", "true"),
        ("screenshot", "true"),
        ("timeout", "1m"),
        ("viewport.width", "1280"),
        ("viewport.height", "640"),
        ("wait_until", "load"),
    ];

    if url.as_str().ends_with(".pdf") {
        params.push(("waitForTimeout", "5000"));
    }

    let response = http_client
        .get(Url::parse_with_params("https://api.microlink.io/", params).unwrap())
        .send()
        .await?
        .server_error()?;

    match response.status() {
        StatusCode::OK => Ok(Ok(response.json::<Response>().await?.data)),
        _ => Ok(Err(response.json::<Error>().await?)),
    }
}
