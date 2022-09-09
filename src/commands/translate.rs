use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::translate;
use crate::utils::Context;

pub struct Translate;

#[async_trait]
impl Command for Translate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = match &ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("text to translate").await;
                return Ok(());
            }
        };

        let translation = translate::single(ctx.http_client.clone(), text, None, "en").await?;
        ctx.reply(format!("{}: {}", translation.source_language, translation.text)).await?;

        Ok(())
    }
}
