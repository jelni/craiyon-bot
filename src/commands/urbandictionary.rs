use std::error::Error;

use async_trait::async_trait;
use tgbotapi::requests::{ParseMode, SendMessage};

use super::Command;
use crate::urbandictionary;
use crate::utils::Context;

pub struct UrbanDictionary;

#[async_trait]
impl Command for UrbanDictionary {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let word = match ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("word to define").await;
                return Ok(());
            }
        };

        let response =
            if let Ok(Some(definition)) = urbandictionary::define(ctx.http_client, word).await {
                definition.as_markdown()
            } else {
                concat!(
                    "There are no definitions for this word\\.\n",
                    "Be the first to [define it](https://urbandictionary.com/add.php)\\!"
                )
                .to_string()
            };

        ctx.api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text: response,
                parse_mode: Some(ParseMode::MarkdownV2),
                reply_to_message_id: Some(ctx.message.message_id),
                disable_web_page_preview: Some(true),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
