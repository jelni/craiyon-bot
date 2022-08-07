use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct Payload {
    pub expr: String,
}

#[derive(Deserialize)]
pub(super) struct Response {
    pub result: String,
}

#[derive(Deserialize)]
pub(super) struct Error {
    pub error: String,
}
