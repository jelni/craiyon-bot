use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::apis::urbandictionary;
use crate::utils::Context;

#[derive(Default)]
pub struct UrbanDictionary;

#[async_trait]
impl CommandTrait for UrbanDictionary {
    fn name(&self) -> &str {
        "urbandictionary"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let word = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("word to define").await;
            return Ok(());
        };

        let response = if let Ok(Some(definition)) =
            urbandictionary::define(ctx.http_client.clone(), word).await
        {
            definition.into_markdown()
        } else {
            concat!(
                "There are no definitions for this word\\.\n",
                "Be the first to [define it](https://urbandictionary.com/add.php)\\!"
            )
            .to_string()
        };
        ctx.reply_markdown(response).await?;

        Ok(())
    }
}
