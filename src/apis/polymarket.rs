use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    #[expect(dead_code)]
    NotFound(Vec<()>),
    Ok {
        events: Vec<Event>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub slug: String,
    pub title: String,
    #[serde(with = "time::serde::iso8601")]
    pub end_date: OffsetDateTime,
    pub markets: Vec<Market>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<String>,
    pub group_item_title: Option<String>,
}

pub async fn search_events(
    http_client: &reqwest::Client,
    query: &str,
) -> reqwest::Result<Option<Vec<Event>>> {
    let response = http_client
        .get(
            Url::parse_with_params("https://polymarket.com/api/events/global", [("q", query)])
                .unwrap(),
        )
        .send()
        .await?;

    let response = response.json::<Response>().await?;

    match response {
        Response::NotFound(_) => Ok(None),
        Response::Ok { events } => Ok(Some(events)),
    }
}
