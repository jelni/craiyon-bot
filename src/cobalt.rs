use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
    pub status: String,
    pub text: Option<String>,
    pub url: Option<String>,
}

pub async fn download<S: AsRef<str>>(
    http_client: reqwest::Client,
    url: S,
) -> reqwest::Result<Result<String, String>> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://co.wukko.me/api/json",
                [
                    // ("audioFormat", "best"),
                    // ("quality", "max"),
                    // ("format", "mp4"),
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
        "success" | "error" => Ok(Err(response.text.unwrap())),
        _ => Ok(Err(format!("unknown status {:?}", response.status))),
    }
}
