use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    pub status: String,
    pub text: Option<String>,
    pub url: Option<String>,
}

pub struct Download {
    pub media: bytes::Bytes,
    pub filename: String,
}

pub async fn query<S: AsRef<str>>(
    http_client: reqwest::Client,
    url: S,
) -> reqwest::Result<Result<String, String>> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://co.wukko.me/api/json",
                [
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
        "stream" | "redirect" => Ok(Ok(response.url.unwrap())),
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

    Ok(Download {
        filename: filename.to_string(),
        media: response.bytes().await?,
    })
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
