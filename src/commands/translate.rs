use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{SourceTargetLanguages, StringGreedyOrReply};
use crate::utilities::google_translate;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::text_utils::EscapeMarkdown;

pub struct Translate;

#[async_trait]
impl CommandTrait for Translate {
    fn command_names(&self) -> &[&str] {
        &["translate", "tr", "trans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("translate text using Google Translate")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let (SourceTargetLanguages(source_language, target_language), StringGreedyOrReply(text)) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        let translation =
            translate::single(ctx.http_client.clone(), text, source_language, &target_language)
                .await?;

        let source_language = EscapeMarkdown(
            google_translate::get_language_name(&translation.source_language).unwrap(),
        );

        let target_language =
            EscapeMarkdown(google_translate::get_language_name(&target_language).unwrap());

        ctx.reply_markdown(format!(
            "*{source_language}* ➜ *{target_language}*\n{}",
            EscapeMarkdown(&translation.text)
        ))
        .await?;

        Ok(())
    }
}
