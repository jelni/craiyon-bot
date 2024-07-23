use url::Url;

use crate::commands::CommandError;

pub async fn petittube() -> Result<Url, CommandError> {
    let url = "https://petittube.com/";
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();

    // Search for the video URL in the HTML content
    // https://www.youtube.com/embed/ is the prefix of the video URL
    let video_split: Vec<&str> = body
        .split("https://www.youtube.com/embed/")
        .collect::<Vec<&str>>();

    // Up until the '?'
    let video = video_split[1].split('?').collect::<Vec<&str>>()[0];

    // Construct the complete YouTube video URL
    let video = "https://www.youtube.com/watch?v=".to_string() + video; 
    let video = Url::parse(&video);

    let res = match video {
        Ok(video) => Ok(video),
        Err(_) => Err(CommandError::Custom("Failed to parse video URL".to_string())),
    };

    res
}