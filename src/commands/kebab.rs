use std::sync::Arc;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::rate_limit::RateLimiter;

const WORDS: [&str; 7] = ["kebab", "king", "house", "super", "arab", "hot", "sauce"];

#[derive(Default)]
pub struct Kebab;

#[async_trait]
impl CommandTrait for Kebab {
    fn command_names(&self) -> &[&str] {
        &["kebab"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("generates a generic kebab shop name")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(10, 30)
    }

    async fn execute(&self, ctx: Arc<CommandContext>, _: Option<String>) -> CommandResult {
        let random_name = WORDS
            .choose_multiple(&mut rand::thread_rng(), 2)
            .copied()
            .collect::<Vec<&str>>()
            .join(" ");

        ctx.reply(random_name).await?;

        Ok(())
    }
}
