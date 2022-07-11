use std::env;

use super::models::{Config, Response};

pub async fn complete_code(
    http_client: reqwest::Client,
    config: Config,
) -> reqwest::Result<String> {
    let text = http_client
        .post("https://api.openai.com/v1/engines/code-davinci-002/completions")
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_TOKEN").unwrap()),
        )
        .json(&config)
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .choices
        .swap_remove(0)
        .text;

    Ok(text)
}
