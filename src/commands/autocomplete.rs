use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::google;
use crate::utils::Context;

#[derive(Default)]
pub struct Autocomplete;

#[async_trait]
impl CommandTrait for Autocomplete {
    fn name(&self) -> &'static str {
        "autocomplete"
    }

    fn aliases(&self) -> &[&str] {
        &["complete", "google"]
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let query = arguments.ok_or(MissingArgument("text to autocomplete"))?;

        let completions = google::complete(ctx.http_client.clone(), &query).await?;
        let query_lowercase = query.to_lowercase();
        ctx.reply_html(
            completions.into_iter().find(|c| *c != query_lowercase).ok_or("no autocompletions")?,
        )
        .await?;

        Ok(())
    }
}
