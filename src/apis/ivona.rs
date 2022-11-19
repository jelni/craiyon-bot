use reqwest::Url;

pub async fn synthesize<S: AsRef<str>>(
    http_client: reqwest::Client,
    text: S,
    voice: &str,
) -> reqwest::Result<Vec<u8>> {
    let mut url =
        Url::parse_with_params("https://ivona.sakamoto.pl/ivonaapi", [("text", text.as_ref())])
            .unwrap();
    url.path_segments_mut().unwrap().push(voice);
    Ok(http_client.get(url).send().await?.error_for_status()?.bytes().await?.to_vec())
}
