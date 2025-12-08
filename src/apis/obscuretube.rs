use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::commands::CommandError;

const OBSCURETUBE_URL: &str = "https://obscuretube.com/obscure.json";

#[derive(Deserialize)]
struct ObscureTubeResponse {
    #[serde(default)]
    data: Vec<ObscureTubeVideo>,
}

#[derive(Deserialize)]
struct ObscureTubeVideo {
    id: String,
}

pub async fn random_video(http_client: &reqwest::Client) -> Result<String, CommandError> {
    let response: ObscureTubeResponse =
        http_client.get(OBSCURETUBE_URL).send().await?.error_for_status()?.json().await?;

    let Some(video) = response.data.choose(&mut rand::rng()) else {
        return Err(CommandError::from("no ObscureTube videos available right now"));
    };

    Ok(video.id.clone())
}
