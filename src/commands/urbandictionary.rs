use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::{CustomMarkdownError, MissingArgument};
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
            Err(CustomMarkdownError(
                concat!(
                    "There are no definitions for this word\\.\n",
                    "Be the first to [define it](https://urbandictionary.com/add.php)\\!"
                )
                .to_string(),
            ))?;
        };

        Ok(())
    }
}
