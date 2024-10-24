use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use crate::commands::cobalt_download::COBALT_URL;
use crate::utilities::api_utils::{DetectServerError, ServerError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    url: &'a str,
    video_quality: &'a str,
    audio_format: &'a str,
    download_mode: &'a str,
    #[serde(rename = "tiktokH265")]
    tiktok_h265: bool,
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
    Tunnel,
}

#[derive(Deserialize)]
struct PickerItem {
    url: String,
}

pub enum Error {
    Cobalt(String),
    Server(ServerError),
    Network(reqwest::Error),
}

pub async fn query(
    http_client: reqwest::Client,
    url: &str,
    audio_only: bool,
) -> Result<Vec<String>, Error> {
    let response = http_client
        .post(COBALT_URL)
        .json(&Payload {
            url,
            video_quality: "1080",
            audio_format: "best",
            download_mode: if audio_only { "audio" } else { "auto" },
            tiktok_h265: true,
        })
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(Error::Network)?
        .server_error()
        .map_err(Error::Server)?
        .json::<Response>()
        .await
        .map_err(Error::Network)?;

    match response.status {
        Status::Stream | Status::Redirect | Status::Tunnel => Ok(vec![response.url.unwrap()]),
        Status::Picker => Ok(response.picker.unwrap().into_iter().map(|i| i.url).collect()),
        Status::Success | Status::Error | Status::RateLimit => {
            Err(Error::Cobalt(response.text.unwrap_or_else(String::new)))
        }
    }
}
