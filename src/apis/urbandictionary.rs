use std::fmt::Write;

use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use time::macros::format_description;
use time::OffsetDateTime;

use crate::utils::escape_markdown;

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

async fn search<S: AsRef<str>>(http_client: reqwest::Client, term: S) -> reqwest::Result<String> {
    let response = http_client
        .get(
            Url::parse_with_params(
                "https://www.urbandictionary.com/define.php",
                [("term", term.as_ref())],
            )
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

            Ok(term.into())
        }
        StatusCode::OK => Ok(term.as_ref().into()),
        _ => unreachable!(),
    }
}

pub async fn define<S: AsRef<str>>(
    http_client: reqwest::Client,
    term: S,
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
        writeln!(result, "[*{}*]({})", escape_markdown(self.word), self.permalink).unwrap();
        writeln!(result, "{}\n", escape_markdown(self.definition)).unwrap();
        if !self.example.is_empty() {
            writeln!(result, "_{}_\n", escape_markdown(self.example)).unwrap();
        }
        writeln!(
            result,
            "by [{}]({}), {}",
            escape_markdown(&self.author),
            Url::parse_with_params(
                "https://urbandictionary.com/author.php",
                [("author", &self.author)]
            )
            .unwrap(),
            escape_markdown(
                self.written_on.format(format_description!("[year]-[month]-[day]")).unwrap()
            )
        )
        .unwrap();
        write!(result, "👍 {} 👎 {}", self.thumbs_up, self.thumbs_down).unwrap();

        result
    }
}
