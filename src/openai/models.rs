use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Config {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub stop: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Response {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub(super) struct Choice {
    pub text: String,
}
