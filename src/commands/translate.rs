use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::Message;
use tdlib::functions;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::{google_translate, telegram_utils};

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

    async fn execute(&self, ctx: Arc<CommandContext>, _: Option<String>) -> CommandResult {
        let text = telegram_utils::get_message_text(&ctx.message)
            .unwrap()
            .chars()
            .skip_while(|char| !char.is_ascii_whitespace())
            .skip_while(char::is_ascii_whitespace)
            .collect::<String>();

        let (source_language, translation_language, text) = google_translate::parse_command(&text);
        let translation_language = translation_language.unwrap_or(&ctx.user.language_code);
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
            translate::single(ctx.http_client.clone(), text, source_language, translation_language)
                .await?;

        let source_language = google_translate::get_language_name(&translation.source_language)
            .unwrap_or(&translation.source_language);
        let translation_language = google_translate::get_language_name(translation_language)
            .unwrap_or(translation_language);

        ctx.reply(format!("{source_language} ➜ {translation_language}\n{}", translation.text))
            .await?;

        Ok(())
    }
}

struct MissingTextToTranslate;

impl From<MissingTextToTranslate> for CommandError {
    fn from(_: MissingTextToTranslate) -> Self {
        CommandError::MissingArgument("text to translate")
    }
}
