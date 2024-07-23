use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::moveit;
use crate::utilities::command_context::CommandContext;

pub struct MoveitJoke;

#[async_trait]
impl CommandTrait for MoveitJoke {
    fn command_names(&self) -> &[&str] {
        &["moveit_joke", "moveit", "muwit"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let joke = moveit::joke(ctx.bot_state.http_client.clone()).await?;
        ctx.reply(format!("[{}] {}", joke.id, joke.joke)).await?;

        Ok(())
    }
}
