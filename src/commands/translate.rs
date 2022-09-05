use std::error::Error;

use async_trait::async_trait;
use tgbotapi::requests::SendMessage;

use super::Command;
use crate::translate;
use crate::utils::Context;

pub struct Translate;

#[async_trait]
impl Command for Translate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let text = match ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("text to translate").await;
                return Ok(());
            }
        };

        let translation = translate::single(ctx.http_client, text, None, "en").await?;

        ctx.api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text: format!("{}: {}", translation.source_language, translation.text),
                reply_to_message_id: Some(ctx.message.message_id),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
