use crate::{commands::CommandError, utilities::api_utils::DetectServerError};

pub async fn random_video(http_client: &reqwest::Client) -> Result<String, CommandError> {
    // This requires web scraping, because the site doesn't have an API
    let url = "https://petittube.com";
    let body = http_client.get(url).send().await?.server_error()?.text().await?;
    let video_split: Vec<&str> = body
        .split("https://www.youtube.com/embed/")
        .collect::<Vec<&str>>();
    let identifier = video_split[1].split('?').collect::<Vec<&str>>()[0];
    let url = format!("https://youtu.be/{}", identifier); 

    Ok(url)
}
