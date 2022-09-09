use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::translate;
use crate::utils::Context;

pub struct BadTranslate;

#[async_trait]
impl Command for BadTranslate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = match &ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("text to translate").await;
                return Ok(());
            }
        };

        let translations =
            translate::multiple(ctx.http_client.clone(), text.split_ascii_whitespace(), None, "en")
                .await?;
        ctx.reply(translations.join(" ")).await?;

        Ok(())
    }
}
