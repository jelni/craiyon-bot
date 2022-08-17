use std::fmt;

use reqwest::{StatusCode, Url};
use serde::Deserialize;
use teloxide::utils::markdown;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

const DATETIME_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");

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
            let (_, term) = response.headers()["Location"]
                .to_str()
                .unwrap()
                .split_once("?term=")
                .unwrap();

            Ok(term.to_string())
        }
        StatusCode::OK => Ok(term.as_ref().to_string()),
        _ => unreachable!(),
    }
}

pub async fn define<S: AsRef<str>>(
    http_client: reqwest::Client,
    term: S,
) -> reqwest::Result<Option<Definition>> {
    let term = search(http_client.clone(), term).await?;
    let mut definitions = http_client
        .get(
            Url::parse_with_params(
                "https://api.urbandictionary.com/v0/define",
                [("term", term)],
            )
            .unwrap(),
        )
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .list;

    if definitions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(definitions.swap_remove(0)))
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[*{}*]({})",
            markdown::escape(&self.word),
            self.permalink
        )?;
        let definition = remove_brackets(self.definition.clone());
        writeln!(f, "{}\n", markdown::escape(&definition))?;
        let example = remove_brackets(self.example.clone());
        writeln!(f, "_{}_\n", markdown::escape(&example))?;
        writeln!(
            f,
            "by [{}]({}), {}",
            markdown::escape(&self.author),
            Url::parse_with_params(
                "https://urbandictionary.com/author.php",
                [("author", &self.author)]
            )
            .unwrap(),
            markdown::escape(&self.written_on.format(DATETIME_FORMAT).unwrap())
        )?;
        write!(
            f,
            "\u{1F44D} {} \u{1F44E} {}",
            self.thumbs_up, self.thumbs_down
        )?;

        Ok(())
    }
}

fn remove_brackets<S: Into<String>>(text: S) -> String {
    let mut text = text.into();
    text.retain(|c| !"[]".contains(c));
    text
}
