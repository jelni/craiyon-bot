use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::apis::google;
use crate::utils::Context;

#[derive(Default)]
pub struct Autocomplete;

#[async_trait]
impl CommandTrait for Autocomplete {
    fn name(&self) -> &str {
        "autocomplete"
    }

    fn aliases(&self) -> &[&str] {
        &["complete", "google"]
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let query = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("text to autocomplete").await;
            return Ok(());
        };

        let completions = google::complete(ctx.http_client.clone(), &query).await?;
        let query_lowercase = query.to_lowercase();
        ctx.reply_html(
            completions
                .into_iter()
                .find(|c| *c != query_lowercase)
                .unwrap_or_else(|| "no autocompletions".to_string()),
        )
        .await?;

        Ok(())
    }
}
