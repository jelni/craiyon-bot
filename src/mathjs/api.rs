use reqwest::StatusCode;

use super::models::{Error, Payload, Response};

pub async fn evaluate<S: Into<String>>(
    http_client: reqwest::Client,
    expr: S,
) -> reqwest::Result<Result<String, String>> {
    let response = http_client
        .post("https://api.mathjs.org/v4/")
        .json(&Payload { expr: expr.into() })
        .send()
        .await?;

    let result = match response.status() {
        StatusCode::OK => Ok(response.json::<Response>().await?.result),
        StatusCode::BAD_REQUEST => Err(response.json::<Error>().await?.error),
        _ => unreachable!(),
    };

    Ok(result)
}
