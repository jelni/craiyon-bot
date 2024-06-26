use std::time::Duration;

use reqwest::StatusCode;

pub async fn status(http_client: reqwest::Client) -> reqwest::Result<StatusCode> {
    Ok(http_client
        .get("https://kiwifarms.st/")
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .status())
}
