use async_trait::async_trait;
use tdlib::types::FormattedText;
use time::macros::format_description;
use url::Url;

use super::{CommandResult, CommandTrait};
use crate::apis::urbandictionary::{self, Card};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{self, ToEntity, ToEntityOwned, ToNestedEntity};

pub struct UrbanDictionary;

#[async_trait]
impl CommandTrait for UrbanDictionary {
    fn command_names(&self) -> &[&str] {
        &["urbandictionary", "urban_dictionary", "ud", "urban", "dictionary"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get a word definition from Urban Dictionary")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(word) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        if let Ok(Some(definition)) =
            urbandictionary::define(ctx.bot_state.http_client.clone(), &word).await
        {
            ctx.reply_formatted_text(format_definition(definition)).await?;
        } else {
            Err("sorry, there are no definitions for this word.")?;
        };

        Ok(())
    }
}

fn format_definition(definition: Card) -> FormattedText {
    let mut description = definition.definition;
    let mut example = definition.example;

    description.retain(|c| !['[', ']'].contains(&c));
    example.retain(|c| !['[', ']'].contains(&c));

    let mut entities = vec![
        definition.word.bold().text_url(definition.permalink),
        "\n".text(),
        description.text(),
        "\n\n".text(),
    ];

    if !example.is_empty() {
        entities.extend([example.italic(), "\n\n".text()]);
    }

    entities.extend([
        "by ".text(),
        definition.author.text_url(
            Url::parse_with_params(
                "https://urbandictionary.com/author.php",
                [("author", &definition.author)],
            )
            .unwrap()
            .to_string(),
        ),
        ", ".text(),
        definition
            .written_on
            .format(format_description!("[year]-[month]-[day]"))
            .unwrap()
            .text_owned(),
        "\n".text(),
    ]);

    entities.extend([
        "👍 ".text(),
        definition.thumbs_up.to_string().text_owned(),
        " 👎 ".text(),
        definition.thumbs_down.to_string().text_owned(),
    ]);

    message_entities::formatted_text(entities)
}
