use std::time::Duration;

use reqwest::StatusCode;

pub async fn status(http_client: reqwest::Client) -> reqwest::Result<StatusCode> {
    Ok(http_client
        .get("https://kiwifarms.net/")
        .timeout(Duration::from_secs(20))
        .send()
        .await?
        .status())
}
