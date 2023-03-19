use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::urbandictionary;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedyOrReply;
use crate::utilities::parse_arguments::ParseArguments;

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
        let StringGreedyOrReply(word) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        ctx.send_typing().await?;

        if let Ok(Some(definition)) = urbandictionary::define(ctx.http_client.clone(), word).await {
            ctx.reply_markdown(definition.into_markdown()).await?;
        } else {
            Err("sorry, there are no definitions for this word.")?;
        };

        Ok(())
    }
}
