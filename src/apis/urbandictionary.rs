use std::borrow::Cow;
use std::fmt::Write;

use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use time::macros::format_description;
use time::OffsetDateTime;

use crate::utilities::text_utils::EscapeMarkdown;

#[derive(Deserialize)]
struct Response {
    pub list: Vec<Definition>,
}

#[derive(Deserialize)]
pub struct Definition {
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

async fn search(http_client: reqwest::Client, term: &str) -> reqwest::Result<Cow<str>> {
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

pub async fn define(
    http_client: reqwest::Client,
    term: &str,
) -> reqwest::Result<Option<Definition>> {
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

impl Definition {
    pub fn into_markdown(mut self) -> String {
        self.definition.retain(|c| !['[', ']'].contains(&c));
        self.example.retain(|c| !['[', ']'].contains(&c));

        let mut result = String::new();
        writeln!(result, "[*{}*]({})", EscapeMarkdown(&self.word), self.permalink).unwrap();
        writeln!(result, "{}\n", EscapeMarkdown(&self.definition)).unwrap();
        if !self.example.is_empty() {
            writeln!(result, "_{}_\n", EscapeMarkdown(&self.example)).unwrap();
        }
        writeln!(
            result,
            "by [{}]({}), {}",
            EscapeMarkdown(&self.author),
            Url::parse_with_params(
                "https://urbandictionary.com/author.php",
                [("author", &self.author)]
            )
            .unwrap(),
            EscapeMarkdown(
                &self.written_on.format(format_description!("[year]-[month]-[day]")).unwrap()
            )
        )
        .unwrap();
        write!(result, "👍 {} 👎 {}", self.thumbs_up, self.thumbs_down).unwrap();

        result
    }
}
