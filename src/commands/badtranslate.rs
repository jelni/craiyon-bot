use std::error::Error;

use async_trait::async_trait;
use tgbotapi::requests::SendMessage;

use super::Command;
use crate::translate;
use crate::utils::Context;

pub struct BadTranslate;

#[async_trait]
impl Command for BadTranslate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let text = match ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("text to translate").await;
                return Ok(());
            }
        };

        let translations =
            translate::multiple(ctx.http_client, text.split_ascii_whitespace(), None, "en").await?;

        let text = translations.join(" ");

        ctx.api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text,
                reply_to_message_id: Some(ctx.message.message_id),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
