use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::petittube;
use crate::utilities::command_context::CommandContext;

pub struct Petittube;

#[async_trait]
impl CommandTrait for Petittube {
    fn command_names(&self) -> &[&str] {
        &["petittube", "noviews"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get a random YouTube video with almost no views")
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let identifier = petittube::random_video(&ctx.bot_state.http_client).await?;
        ctx.reply_webpage(format!("https://youtu.be/{identifier}")).await?;

        Ok(())
    }
}
