use reqwest::header::{ACCEPT, CONTENT_DISPOSITION};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload<'a> {
    url: &'a str,
    v_format: &'a str,
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

pub struct Download {
    pub media: Vec<u8>,
    pub filename: String,
}

pub async fn query<S: AsRef<str>>(
    http_client: reqwest::Client,
    url: S,
) -> reqwest::Result<Result<Vec<String>, String>> {
    let response = http_client
        .post("https://co.wukko.me/api/json")
        .json(&Payload {
            url: url.as_ref(),
            v_format: "mp4",
            v_quality: "max",
            a_format: "best",
            is_no_ttwatermark: true,
        })
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Response>()
        .await?;

    match response.status {
        Status::Stream | Status::Redirect => Ok(Ok(vec![response.url.unwrap()])),
        Status::Picker => Ok(Ok(response.picker.unwrap().into_iter().map(|i| i.url).collect())),
        Status::Success | Status::Error | Status::RateLimit => Ok(Err(response.text.unwrap())),
    }
}

pub async fn download<S: AsRef<str>>(
    http_client: reqwest::Client,
    url: S,
) -> reqwest::Result<Download> {
    let response = http_client.get(url.as_ref()).send().await?;

    let filename = match response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|h| parse_filename(h.to_str().unwrap()))
    {
        Some(filename) => filename,
        None => response.url().path_segments().unwrap().last().unwrap(),
    };

    Ok(Download { filename: filename.to_string(), media: response.bytes().await?.to_vec() })
}

/// parses the `filename` from a `Content-Disposition` header
fn parse_filename(value: &str) -> Option<&str> {
    value.split(';').find_map(|dir| {
        let mut pair = dir.trim().split('=');
        if pair.next().unwrap() == "filename" {
            Some(pair.next().unwrap().trim_matches('"'))
        } else {
            None
        }
    })
}
