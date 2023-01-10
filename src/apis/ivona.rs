use reqwest::Url;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;

pub async fn synthesize<S: AsRef<str>>(
    http_client: reqwest::Client,
    text: S,
    voice: &str,
) -> Result<Vec<u8>, CommandError> {
    let mut url =
        Url::parse_with_params("https://ivona.sakamoto.pl/ivonaapi", [("text", text.as_ref())])
            .unwrap();
    url.path_segments_mut().unwrap().push(voice);
    Ok(http_client.get(url).send().await?.server_error()?.bytes().await?.to_vec())
}
