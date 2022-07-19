use std::fmt;

use reqwest::Url;
use serde::Deserialize;
use teloxide::utils::markdown;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

const DATETIME_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");

#[derive(Deserialize)]
pub struct Response {
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
                "https://www.urbandictionary.com/author.php",
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
