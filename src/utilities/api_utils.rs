use reqwest::header::CONTENT_TYPE;
use reqwest::{Response, StatusCode};

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
