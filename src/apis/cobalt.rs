use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    pub status: String,
    pub text: Option<String>,
    pub url: Option<Urls>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Urls {
    Single(String),
    Many(Vec<Media>),
}

#[derive(Deserialize)]
struct Media {
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
        .get(
            Url::parse_with_params(
                "https://co.wukko.me/api/json",
                [
                    ("format", "mp4"),
                    ("nw", "true"), // no TikTok watermark
                    ("url", url.as_ref()),
                ],
            )
            .unwrap(),
        )
        .send()
        .await?
        .json::<Response>()
        .await?;

    match response.status.as_str() {
        "stream" | "redirect" | "picker" => {
            let urls = match response.url.unwrap() {
                Urls::Single(url) => vec![url],
                Urls::Many(media) => media.into_iter().map(|m| m.url).collect(),
            };
            Ok(Ok(urls))
        }
        "success" | "error" | "rate-limit" => Ok(Err(response.text.unwrap())),
        _ => Ok(Err(format!("unknown status: {:?}", response.status))),
    }
}

pub async fn download<S: AsRef<str>>(
    http_client: reqwest::Client,
    url: S,
) -> reqwest::Result<Download> {
    let response = http_client.get(url.as_ref()).send().await?;

    let filename = match response
        .headers()
        .get("Content-Disposition")
        .and_then(|h| parse_filename(h.to_str().unwrap()))
    {
        Some(filename) => filename,
        None => response.url().path_segments().unwrap().last().unwrap(),
    };

    Ok(Download { filename: filename.to_string(), media: response.bytes().await?.to_vec() })
}

/// parses the `filename` from a `Content-Disposition` header
fn parse_filename(header: &str) -> Option<&str> {
    header.split(';').find_map(|dir| {
        let mut pair = dir.trim().split('=');
        if pair.next().unwrap() == "filename" {
            Some(pair.next().unwrap().trim_matches('"'))
        } else {
            None
        }
    })
}
