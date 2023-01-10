use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Payload {
    pub expr: String,
}

#[derive(Deserialize)]
struct Response {
    pub result: String,
}

#[derive(Deserialize)]
struct Error {
    pub error: String,
}

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
        _ => Err(response.json::<Error>().await?.error),
    };

    Ok(result)
}
