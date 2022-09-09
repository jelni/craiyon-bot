use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::urbandictionary;
use crate::utils::Context;

pub struct UrbanDictionary;

#[async_trait]
impl Command for UrbanDictionary {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let word = match &ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("word to define").await;
                return Ok(());
            }
        };

        let response = if let Ok(Some(definition)) =
            urbandictionary::define(ctx.http_client.clone(), word).await
        {
            definition.as_markdown()
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
