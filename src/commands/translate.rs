use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::Message;
use tdlib::functions;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::google_translate::MissingTextToTranslate;
use crate::utilities::{google_translate, telegram_utils, text_utils};

#[derive(Default)]
pub struct Translate;

#[async_trait]
impl CommandTrait for Translate {
    fn command_names(&self) -> &[&str] {
        &["translate", "tr", "trans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("translate text using Google Translate")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingTextToTranslate)?;

        let (source_language, target_language, text) = google_translate::parse_command(&text);
        let target_language = target_language.unwrap_or(&ctx.user.language_code);
        let mut text = text.to_owned();

        if text.is_empty() {
            if ctx.message.reply_to_message_id == 0 {
                Err(MissingTextToTranslate)?;
            }

            let Message::Message(message) = functions::get_message(
                ctx.message.chat_id,
                ctx.message.reply_to_message_id,
                ctx.client_id,
            )
            .await?;

            text = telegram_utils::get_message_text(&message).ok_or(MissingTextToTranslate)?;
        }

        let translation =
            translate::single(ctx.http_client.clone(), text, source_language, target_language)
                .await?;

        let source_language = google_translate::get_language_name(&translation.source_language)
            .unwrap_or(&translation.source_language);
        let target_language =
            google_translate::get_language_name(target_language).unwrap_or(target_language);

        ctx.reply_markdown(format!(
            "*{source_language}* ➜ *{target_language}*\n{}",
            text_utils::escape_markdown(translation.text)
        ))
        .await?;

        Ok(())
    }
}
