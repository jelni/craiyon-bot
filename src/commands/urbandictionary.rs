use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::urbandictionary;
use crate::utilities::command_context::CommandContext;

#[derive(Default)]
pub struct UrbanDictionary;

#[async_trait]
impl CommandTrait for UrbanDictionary {
    fn command_names(&self) -> &[&str] {
        &["urbandictionary", "urban_dictionary", "ud", "urban", "dictionary"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get a word definition from Urban Dictionary")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let word = arguments.ok_or(MissingArgument("word to define"))?;

        ctx.send_typing().await?;

        if let Ok(Some(definition)) = urbandictionary::define(ctx.http_client.clone(), word).await {
            ctx.reply_markdown(definition.into_markdown()).await?;
        } else {
            Err("sorry, there are no definitions for this word.")?;
        };

        Ok(())
    }
}
