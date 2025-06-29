use std::collections::HashMap;

use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utilities::api_utils::{DetectServerError, ServerError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    url: &'a str,
    video_quality: &'a str,
    audio_format: &'a str,
    download_mode: &'a str,
    tiktok_full_audio: bool,
    twitter_gif: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "status")]
pub enum Response {
    Redirect(File),
    Tunnel(File),
    Picker(Picker),
    Error(CobaltError),
}

#[derive(Deserialize)]
pub struct File {
    pub url: String,
    pub filename: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Picker {
    pub audio: Option<String>,
    pub audio_filename: Option<String>,
    #[expect(clippy::struct_field_names)]
    pub picker: Vec<PickerItem>,
}

#[derive(Deserialize)]
pub struct PickerItem {
    pub url: String,
    pub thumb: Option<String>,
}

#[expect(clippy::module_name_repetitions)]
#[derive(Deserialize)]
pub struct CobaltError {
    pub error: ErrorContext,
}

#[derive(Deserialize)]
pub struct ErrorContext {
    pub code: String,
    pub context: Option<HashMap<String, Value>>,
}

pub enum Error {
    Server(ServerError),
    Network(reqwest::Error),
}

pub async fn query(
    http_client: &reqwest::Client,
    instance: &str,
    api_key: Option<&str>,
    url: &str,
    audio_only: bool,
) -> Result<Response, Error> {
    let mut request = http_client
        .post(instance)
        .json(&Payload {
            url,
            video_quality: "1080",
            audio_format: "best",
            download_mode: if audio_only { "audio" } else { "auto" },
            tiktok_full_audio: true,
            twitter_gif: false,
        })
        .header(ACCEPT, "application/json");

    if let Some(api_key) = api_key {
        request = request.header(AUTHORIZATION, format!("Api-Key {api_key}"));
    }

    let response =
        request.send().await.map_err(Error::Network)?.server_error().map_err(Error::Server)?;

    response.json::<Response>().await.map_err(Error::Network)
}

pub async fn get_api_error_localization(
    http_client: &reqwest::Client,
) -> reqwest::Result<HashMap<String, String>> {
    let request = http_client
        .get(concat!(
            "https://raw.githubusercontent.com",
            "/imputnet/cobalt/refs/heads/main/web/i18n/en/error/api.json"
        ))
        .send()
        .await?;

    request.json().await
}
