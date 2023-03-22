use bytes::Bytes;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Response, StatusCode, Url};
use url::ParseError;

const CLOUDFLARE_STORAGE: &str = "r2.cloudflarestorage.com";

pub struct ServerError(pub StatusCode);

pub trait DetectServerError {
    fn server_error(self) -> Result<Response, ServerError>;
}

impl DetectServerError for Response {
    fn server_error(self) -> Result<Response, ServerError> {
        if self.status().is_server_error()
            && self.headers().get(CONTENT_TYPE).map_or(false, |header| {
                header.to_str().map_or(false, |header| header.starts_with("text/html"))
            })
        {
            return Err(ServerError(self.status()));
        }

        Ok(self)
    }
}

pub enum InvalidCloudflareStorageUrl {
    ParseError(ParseError),
    InvalidDomain,
}

pub fn cloudflare_storage_url(url: &str) -> Result<Url, InvalidCloudflareStorageUrl> {
    Url::parse(url).map_err(InvalidCloudflareStorageUrl::ParseError).and_then(|url| {
        if let Some(host) = url.host_str() {
            if host.ends_with(CLOUDFLARE_STORAGE) {
                Ok(url)
            } else {
                Err(InvalidCloudflareStorageUrl::InvalidDomain)
            }
        } else {
            Err(InvalidCloudflareStorageUrl::InvalidDomain)
        }
    })
}

pub async fn simultaneous_download(
    http_client: reqwest::Client,
    urls: impl Iterator<Item = Url>,
) -> reqwest::Result<Vec<Bytes>> {
    let tasks = urls
        .map(|url| {
            let http_client = http_client.clone();
            tokio::spawn(async move {
                let bytes = http_client.get(url).send().await?.bytes().await?;
                reqwest::Result::Ok(bytes)
            })
        })
        .collect::<Vec<_>>();

    let mut downloads = Vec::with_capacity(tasks.len());
    for task in tasks {
        downloads.push(task.await.unwrap()?);
    }

    Ok(downloads)
}
