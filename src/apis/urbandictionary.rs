use std::borrow::Cow;

use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize)]
struct Response {
    pub list: Vec<Card>,
}

#[derive(Deserialize)]
pub struct Card {
    pub word: String,
    pub definition: String,
    pub example: String,
    pub author: String,
    #[serde(with = "time::serde::iso8601")]
    pub written_on: OffsetDateTime,
    pub thumbs_up: usize,
    pub thumbs_down: usize,
    pub permalink: String,
}

async fn search(http_client: reqwest::Client, term: &str) -> reqwest::Result<Cow<'_, str>> {
    let response = http_client
        .get(
            Url::parse_with_params("https://www.urbandictionary.com/define.php", [("term", term)])
                .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?;

    match response.status() {
        StatusCode::FOUND => {
            let term = response
                .headers()
                .get(LOCATION)
                .unwrap()
                .to_str()
                .unwrap()
                .split_once("?term=")
                .unwrap()
                .1;

            Ok(Cow::Owned(term.into()))
        }
        StatusCode::OK => Ok(Cow::Borrowed(term)),
        _ => unreachable!(),
    }
}

pub async fn define(http_client: reqwest::Client, term: &str) -> reqwest::Result<Option<Card>> {
    let term = search(http_client.clone(), term).await?;
    let definitions = http_client
        .get(
            Url::parse_with_params("https://api.urbandictionary.com/v0/define", [("term", term)])
                .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .list;

    Ok(definitions.into_iter().next())
}
