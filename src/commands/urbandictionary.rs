use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::urbandictionary;
use crate::utils::Context;

#[derive(Default)]
pub struct UrbanDictionary;

#[async_trait]
impl CommandTrait for UrbanDictionary {
    fn name(&self) -> &'static str {
        "urbandictionary"
    }

    fn aliases(&self) -> &[&str] {
        &["ud", "urban", "dictionary"]
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let word = arguments.ok_or(MissingArgument("word to define"))?;

        if let Ok(Some(definition)) = urbandictionary::define(ctx.http_client.clone(), word).await {
            ctx.reply_markdown(definition.into_markdown()).await?;
        } else {
            Err("sorry, there are no definitions for this word.")?;
        };

        Ok(())
    }
}
