use std::sync::Arc;

use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::google;
use crate::utilities::command_context::CommandContext;
use crate::utilities::rate_limit::RateLimiter;

#[derive(Default)]
pub struct Autocomplete;

#[async_trait]
impl CommandTrait for Autocomplete {
    fn command_names(&self) -> &[&str] {
        &["autocomplete", "complete", "google"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("autocompletes a query with Google")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(10, 30)
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let query = arguments.ok_or(MissingArgument("text to autocomplete"))?;

        let completions =
            google::complete(ctx.http_client.clone(), &query).await.unwrap_or_default();
        ctx.reply(completions.choose(&mut StdRng::from_entropy()).ok_or("no autocompletions")?)
            .await?;

        Ok(())
    }
}
