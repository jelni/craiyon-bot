use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::apis::translate;
use crate::utils::Context;

#[derive(Default)]
pub struct BadTranslate;

#[async_trait]
impl CommandTrait for BadTranslate {
    fn name(&self) -> &'static str {
        "badtranslate"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(text) = arguments else {
            ctx.missing_argument("text to translate").await;
            return Ok(());
        };

        let translations =
            translate::multiple(ctx.http_client.clone(), text.split_ascii_whitespace(), None, "en")
                .await?;
        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
