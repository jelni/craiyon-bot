use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::obscuretube;
use crate::utilities::command_context::CommandContext;

pub struct ObscureTube;

#[async_trait]
impl CommandTrait for ObscureTube {
    fn command_names(&self) -> &[&str] {
        &["obscuretube", "obscure", "noviews"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get a random obscure YouTube video with very few views")
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let identifier = obscuretube::random_video(&ctx.bot_state.http_client).await?;
        ctx.reply_webpage(format!("https://youtu.be/{identifier}")).await?;

        Ok(())
    }
}
