use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::Message;
use tdlib::functions;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::google_translate::{self, MissingTextToTranslate};
use crate::utilities::telegram_utils;

#[derive(Default)]
pub struct BadTranslate;

#[async_trait]
impl CommandTrait for BadTranslate {
    fn command_names(&self) -> &[&str] {
        &["badtranslate", "btr", "btrans"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("badly translate text by translating every word separately")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingTextToTranslate)?;

        let (source_language, target_language, mut text) = google_translate::parse_command(text);

        let target_language = target_language.unwrap_or(if ctx.user.language_code.is_empty() {
            "en"
        } else {
            &ctx.user.language_code
        });

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

        let translations = translate::multiple(
            ctx.http_client.clone(),
            text.split_ascii_whitespace(),
            source_language,
            target_language,
        )
        .await?;

        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
