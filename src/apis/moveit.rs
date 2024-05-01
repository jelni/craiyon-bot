use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Deserialize)]
pub struct Joke {
    pub id: u32,
    pub joke: String,
}

pub async fn joke(http_client: reqwest::Client) -> Result<Joke, CommandError> {
    let joke = http_client
        .get("https://moveit.ducky.pics/json")
        .send()
        .await?
        .server_error()?
        .json::<Joke>()
        .await?;

    Ok(joke)
}
