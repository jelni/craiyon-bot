use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

const YOUTUBE_EMBED: &str = "https://www.youtube.com/embed/";

pub async fn random_video(http_client: &reqwest::Client) -> Result<String, CommandError> {
    // This requires web scraping, because the site doesn't have an API
    let body =
        http_client.get("https://petittube.com").send().await?.server_error()?.text().await?;
    let index =
        body.find(YOUTUBE_EMBED).ok_or(CommandError::Custom("YOUTUBE_EMBED not found".into()))?;
    let identifier: String =
        body[index + YOUTUBE_EMBED.len()..].chars().take_while(|&c| c != '?').collect();
    let url = format!("https://youtu.be/{identifier}");

    Ok(url)
}
