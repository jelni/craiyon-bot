use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    url: &'a str,
    v_codec: &'a str,
    v_quality: &'a str,
    a_format: &'a str,
    #[serde(rename = "isNoTTWatermark")]
    is_no_ttwatermark: bool,
}

#[derive(Deserialize)]
struct Response {
    status: Status,
    text: Option<String>,
    url: Option<String>,
    picker: Option<Vec<PickerItem>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Status {
    Stream,
    Redirect,
    Picker,
    Success,
    Error,
    RateLimit,
}

#[derive(Deserialize)]
struct PickerItem {
    url: String,
}

pub async fn query(
    http_client: reqwest::Client,
    url: &str,
) -> Result<Result<Vec<String>, String>, CommandError> {
    let response = http_client
        .post("https://co.wuk.sh/api/json")
        .json(&Payload {
            url,
            v_codec: "h264",
            v_quality: "1080",
            a_format: "best",
            is_no_ttwatermark: true,
        })
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .server_error()?
        .json::<Response>()
        .await?;

    match response.status {
        Status::Stream | Status::Redirect => Ok(Ok(vec![response.url.unwrap()])),
        Status::Picker => Ok(Ok(response.picker.unwrap().into_iter().map(|i| i.url).collect())),
        Status::Success | Status::Error | Status::RateLimit => Ok(Err(response.text.unwrap())),
    }
}
