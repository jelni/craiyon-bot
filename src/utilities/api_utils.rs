use reqwest::header::CONTENT_TYPE;
use reqwest::{Response, StatusCode, Url};

const CLOUDFLARE_STORAGE: &str = "r2.cloudflarestorage.com";

pub struct ServerError(pub StatusCode);

pub trait DetectServerError {
    fn server_error(self) -> Result<Response, ServerError>;
}

impl DetectServerError for Response {
    fn server_error(self) -> Result<Response, ServerError> {
        if self.status().is_server_error()
            && self.headers().get(CONTENT_TYPE).is_some_and(|header| {
                header.to_str().is_ok_and(|header| header.starts_with("text/html"))
            })
        {
            return Err(ServerError(self.status()));
        }

        Ok(self)
    }
}

pub enum InvalidCloudflareStorageUrl {
    ParseError,
    InvalidDomain,
}

pub fn cloudflare_storage_url(url: &str) -> Result<Url, InvalidCloudflareStorageUrl> {
    Url::parse(url).map_err(|_| InvalidCloudflareStorageUrl::ParseError).and_then(|url| {
        url.host_str().map_or(Err(InvalidCloudflareStorageUrl::InvalidDomain), |host| {
            if host.ends_with(CLOUDFLARE_STORAGE) {
                Ok(url.clone())
            } else {
                Err(InvalidCloudflareStorageUrl::InvalidDomain)
            }
        })
    })
}
