use std::error::Error;

use async_trait::async_trait;
use tgbotapi::requests::SendMessage;

use super::Command;
use crate::poligon;
use crate::utils::Context;

pub struct StartItJoke;

#[async_trait]
impl Command for StartItJoke {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let joke = poligon::startit_joke(ctx.http_client).await?;

        ctx.api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text: joke,
                reply_to_message_id: Some(ctx.message.message_id),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
