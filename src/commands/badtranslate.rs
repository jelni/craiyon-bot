use std::sync::Arc;

use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{SourceTargetLanguages, StringGreedyOrReply};
use crate::utilities::parse_arguments::ParseArguments;

pub struct BadTranslate;

#[async_trait]
impl CommandTrait for BadTranslate {
    fn command_names(&self) -> &[&str] {
        &["badtranslate", "btr", "btrans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("badly translate text by translating every word separately")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: String) -> CommandResult {
        let (SourceTargetLanguages(source_language, target_language), StringGreedyOrReply(text)) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        let translations = translate::multiple(
            ctx.http_client.clone(),
            text.split_ascii_whitespace(),
            source_language,
            &target_language,
        )
        .await?;

        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
