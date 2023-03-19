use std::sync::Arc;

use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use super::{CommandResult, CommandTrait};
use crate::apis::google;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedyOrReply;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::rate_limit::RateLimiter;

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

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: String) -> CommandResult {
        let StringGreedyOrReply(query) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        let completions =
            google::complete(ctx.http_client.clone(), &query).await.unwrap_or_default();
        ctx.reply(completions.choose(&mut StdRng::from_entropy()).ok_or("no autocompletions")?)
            .await?;

        Ok(())
    }
}
