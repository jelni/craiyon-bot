use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tgbotapi::FileType;

use super::CommandTrait;
use crate::api_methods::SendVoice;
use crate::apis::ivona;
use crate::utils::Context;

#[derive(Default)]
pub struct Tts;

#[async_trait]
impl CommandTrait for Tts {
    fn name(&self) -> &'static str {
        "tts"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(text) = arguments else {
            ctx.missing_argument("text to synthesize").await;
            return Ok(());
        };

        if text.chars().count() > 1024 {
            ctx.reply("This text is too long.").await?;
            return Ok(());
        }

        let bytes = ivona::synthesize(ctx.http_client.clone(), text, "jan").await?;

        ctx.api
            .make_request(&SendVoice {
                chat_id: ctx.message.chat_id(),
                voice: FileType::Bytes("voice.wav".to_string(), bytes),
                reply_to_message_id: Some(ctx.message.message_id),
                allow_sending_without_reply: Some(true),
            })
            .await?;

        Ok(())
    }
}
