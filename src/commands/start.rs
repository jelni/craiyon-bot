use std::error::Error;

use async_trait::async_trait;
use tgbotapi::requests::{ParseMode, SendMessage};

use super::Command;
use crate::utils::Context;

pub struct Start;

#[async_trait]
impl Command for Start {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        ctx.api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text: concat!(
                    "Use the /generate command to generate images\\.\n",
                    "*Example:* `/generate crayons in a box`"
                )
                .to_string(),
                parse_mode: Some(ParseMode::MarkdownV2),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
