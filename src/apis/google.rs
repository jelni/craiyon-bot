use reqwest::Url;
use serde_json::Value;

pub async fn complete<S: AsRef<str>>(
    http_client: reqwest::Client,
    query: S,
) -> reqwest::Result<Vec<String>> {
    let text = http_client
        .get(
            Url::parse_with_params(
                "https://google.com/complete/search",
                [("q", query.as_ref()), ("client", "gws-wiz"), ("xssi", "t")],
            )
            .unwrap(),
        )
        .send()
        .await?
        .text()
        .await?;

    let data = serde_json::from_str::<Value>(text.lines().nth(1).unwrap()).unwrap();

    // parses the funny Google model: [[[completion, ...]], ...]
    let completions = match data {
        Value::Array(value) => match value.into_iter().next() {
            Some(Value::Array(value)) => Some(
                value
                    .into_iter()
                    .filter_map(|value| match value {
                        Value::Array(value) => match value.into_iter().next() {
                            Some(Value::String(text)) => Some(text),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        },
        _ => None,
    };

    Ok(completions.unwrap_or_default())
}
