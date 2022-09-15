use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::Command;
use crate::apis::translate;
use crate::utils::Context;

pub struct Translate;

#[async_trait]
impl Command for Translate {
    fn name(&self) -> &str {
        "translate"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("text to translate").await;
            return Ok(());
        };

        let translation = translate::single(ctx.http_client.clone(), text, None, "en").await?;
        ctx.reply(format!("{}: {}", translation.source_language, translation.text)).await?;

        Ok(())
    }
}
