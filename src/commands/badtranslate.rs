use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::google_translate::SourceTargetLanguages;

pub struct BadTranslate;

#[async_trait]
impl CommandTrait for BadTranslate {
    fn command_names(&self) -> &[&str] {
        &["badtranslate", "btr", "btrans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("badly translate text by translating every word separately")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let (SourceTargetLanguages(source_language, target_language), StringGreedyOrReply(text)) =
            ConvertArgument::convert(ctx, &arguments).await?.0;

        let translations = translate::multiple(
            ctx.bot_state.http_client.clone(),
            &text.split_ascii_whitespace().collect::<Vec<_>>(),
            source_language,
            &target_language,
        )
        .await?;

        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
