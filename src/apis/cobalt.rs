use std::collections::HashMap;

use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use crate::commands::CommandError;
use crate::utilities::api_utils::{DetectServerError, ServerError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    url: &'a str,
    v_quality: &'a str,
    a_format: &'a str,
    is_audio_only: bool,
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
    domain: &str,
    url: &str,
    audio_only: bool,
) -> Result<Vec<String>, Error> {
    let response = http_client
        .post(format!("https://{domain}/api/json"))
        .json(&Payload {
            url,
            v_quality: "1080",
            a_format: "best",
            is_audio_only: audio_only,
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
        Status::Stream | Status::Redirect => Ok(vec![response.url.unwrap()]),
        Status::Picker => Ok(response.picker.unwrap().into_iter().map(|i| i.url).collect()),
        Status::Success | Status::Error | Status::RateLimit => {
            Err(Error::Cobalt(response.text.unwrap()))
        }
    }
}

#[derive(Deserialize)]
pub struct Instance {
    pub api_online: bool,
    pub services: HashMap<Service, bool>,
    pub score: f32,
    pub protocol: String,
    pub api: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Youtube,
    Rutube,
    Tumblr,
    Bilibili,
    Pinterest,
    Instagram,
    Soundcloud,
    YoutubeMusic,
    Odnoklassniki,
    Dailymotion,
    Twitter,
    Vimeo,
    Streamable,
    Vk,
    Tiktok,
    Reddit,
    TwitchClips,
    YoutubeShorts,
    #[serde(untagged)]
    Unknown,
}

impl Service {
    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::Youtube => Some("YouTube"),
            Self::Rutube => Some("RUTUBE"),
            Self::Tumblr => Some("Tumblr"),
            Self::Bilibili => Some("BiliBili"),
            Self::Pinterest => Some("Pinterest"),
            Self::Instagram => Some("Instagram"),
            Self::Soundcloud => Some("SoundCloud"),
            Self::YoutubeMusic => Some("YouTube Music"),
            Self::Odnoklassniki => Some("Odnoklassniki"),
            Self::Dailymotion => Some("Dailymotion"),
            Self::Twitter => Some("Twitter"),
            Self::Vimeo => Some("Vimeo"),
            Self::Streamable => Some("Streamable"),
            Self::Vk => Some("VK"),
            Self::Tiktok => Some("TikTok"),
            Self::Reddit => Some("Reddit"),
            Self::TwitchClips => Some("Twitch (Clips)"),
            Self::YoutubeShorts => Some("YouTube (Shorts)"),
            Self::Unknown => None,
        }
    }
}

pub async fn instances(http_client: reqwest::Client) -> Result<Vec<Instance>, CommandError> {
    let response = http_client
        .get("https://instances.hyper.lol/instances.json")
        .send()
        .await?
        .server_error()?;

    Ok(response.json().await?)
}
