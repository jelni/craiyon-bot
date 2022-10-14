use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tgbotapi::FileType;

use super::CommandTrait;
use crate::api_methods::SendDocument;
use crate::apis::translate;
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
        let text = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("text to synthesize").await;
            return Ok(());
        };

        let language = translate::detect_language(ctx.http_client.clone(), &text).await?;
        let bytes = translate::tts(ctx.http_client.clone(), text, &language).await?;

        ctx.api
            .make_request(&SendDocument {
                chat_id: ctx.message.chat_id(),
                document: FileType::Bytes(format!("tts_{language}.mp3"), bytes),
                reply_to_message_id: Some(ctx.message.message_id),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
