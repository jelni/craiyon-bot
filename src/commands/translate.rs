use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utils::Context;

#[derive(Default)]
pub struct Translate;

#[async_trait]
impl CommandTrait for Translate {
    fn name(&self) -> &'static str {
        "translate"
    }

    fn aliases(&self) -> &[&str] {
        &["tr", "trans"]
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingArgument("text to translate"))?;

        let translation = translate::single(ctx.http_client.clone(), text, None, "en").await?;
        ctx.reply(format!("{}: {}", translation.source_language, translation.text)).await?;

        Ok(())
    }
}
