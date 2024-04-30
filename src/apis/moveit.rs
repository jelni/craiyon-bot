use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Deserialize)]
struct MoveitJoke {
    id: u32,
    joke: String,
}

pub async fn moveit_joke(http_client: reqwest::Client) -> Result<String, CommandError> {
    let joke = http_client
        .get("https://moveit.ducky.pics/json")
        .send()
        .await?
        .server_error()?
        .json::<MoveitJoke>()
        .await?;

    Ok(format!("[{}] {}", joke.id, joke.joke))
}
