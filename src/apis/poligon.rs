use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

#[derive(Deserialize)]
struct StartitJoke {
    joke: String,
}

pub async fn startit_joke(http_client: reqwest::Client) -> Result<String, CommandError> {
    let joke = http_client
        .get("https://astolfo.poligon.lgbt/api/startit")
        .send()
        .await?
        .server_error()?
        .json::<StartitJoke>()
        .await?;

    Ok(joke.joke)
}
