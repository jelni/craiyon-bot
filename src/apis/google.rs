use reqwest::Url;
use serde_json::Value;

pub async fn complete<S: AsRef<str>>(
    http_client: reqwest::Client,
    query: S,
) -> reqwest::Result<Vec<String>> {
    let data = http_client
        .get(
            Url::parse_with_params(
                "https://google.com/complete/search",
                [("q", query.as_ref()), ("client", "chrome")],
            )
            .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<(String, Vec<String>, Value, Value, Value)>()
        .await?;

    Ok(data.1)
}
