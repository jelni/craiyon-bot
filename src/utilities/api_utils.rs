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
