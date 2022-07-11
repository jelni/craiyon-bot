use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct Payload {
    pub prompt: String,
}

#[derive(Deserialize)]
pub(super) struct Response {
    pub images: Vec<String>,
}

pub struct GeneratedResult {
    pub images: Vec<Vec<u8>>,
    pub duration: Duration,
}
