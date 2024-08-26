use crate::commands::CommandError;

const EMBED_URL: &str = "https://www.youtube.com/embed/";

pub async fn random_video(http_client: &reqwest::Client) -> Result<String, CommandError> {
    let body = http_client.get("https://petittube.com/").send().await?.text().await?;
    let index = body.find(EMBED_URL).unwrap();
    let identifier =
        body[index + EMBED_URL.len()..].chars().take_while(|&c| c != '?').collect::<String>();

    Ok(identifier)
}
