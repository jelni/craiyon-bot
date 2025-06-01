use reqwest::header::HOST;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
struct Response {
    events: Vec<Event>,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub title: String,
    pub markets: Vec<Market>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<String>,
    pub group_item_title: Option<String>,
    pub group_item_threshold: Option<String>,
}

pub async fn search_events(
    http_client: &reqwest::Client,
    query: &str,
) -> reqwest::Result<Vec<Event>> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://polymarket.com/api/events/search",
                [("_q", query), ("_p", "1")],
            )
            .unwrap(),
        )
        .header(HOST, "polymarket.com")
        .send()
        .await?;

    Ok(response.json::<Response>().await?.events)
}
