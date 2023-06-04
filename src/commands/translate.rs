use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::google_translate::SourceTargetLanguages;
use crate::utilities::message_entities::ToEntity;
use crate::utilities::{google_translate, message_entities};

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
            ConvertArgument::convert(ctx, &arguments).await?.0;

        let translation =
            translate::single(ctx.http_client.clone(), &text, source_language, &target_language)
                .await?;

        let source_language = google_translate::get_language_name(&translation.source_language)
            .unwrap_or(&translation.source_language);

        let target_language =
            google_translate::get_language_name(&target_language).unwrap_or(&target_language);

        ctx.reply_formatted_text(message_entities::formatted_text(vec![
            source_language.bold(),
            " ➜ ".text(),
            target_language.bold(),
            "\n".text(),
            translation.text.text(),
        ]))
        .await?;

        Ok(())
    }
}
