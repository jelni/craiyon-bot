use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utils::Context;

#[derive(Default)]
pub struct BadTranslate;

#[async_trait]
impl CommandTrait for BadTranslate {
    fn command_names(&self) -> &[&str] {
        &["badtranslate", "btr", "btrans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("badly translate text to English")
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingArgument("text to translate"))?;

        let translations =
            translate::multiple(ctx.http_client.clone(), text.split_ascii_whitespace(), None, "en")
                .await?;
        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
