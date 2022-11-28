use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::command_context::CommandContext;

#[derive(Default)]
pub struct Translate;

#[async_trait]
impl CommandTrait for Translate {
    fn command_names(&self) -> &[&str] {
        &["translate", "tr", "trans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("translate text to English using Google Translate")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingArgument("text to translate"))?;

        let translation = translate::single(ctx.http_client.clone(), text, None, "en").await?;
        ctx.reply(format!("{}: {}", translation.source_language, translation.text)).await?;

        Ok(())
    }
}
