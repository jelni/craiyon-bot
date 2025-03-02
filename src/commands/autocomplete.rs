use async_trait::async_trait;
use rand::seq::IteratorRandom;

use super::{CommandResult, CommandTrait};
use crate::apis::google;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
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

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(query) = ConvertArgument::convert(ctx, &arguments).await?.0;

        let completions =
            google::complete(ctx.bot_state.http_client.clone(), &query).await.unwrap_or_default();

        let completion = completions.into_iter().choose(&mut rand::rng());

        ctx.reply(completion.ok_or("no autocompletions")?).await?;

        Ok(())
    }
}
