use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::opera_gx;
use crate::utilities::command_context::CommandContext;

pub struct Nitro;

#[async_trait]
impl CommandTrait for Nitro {
    fn command_names(&self) -> &[&str] {
        &["nitro"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("Generates a Discord Nitro promotional link")
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let link = opera_gx::generate(ctx.bot_state.http_client.clone()).await?;

        ctx.reply(link).await?;

        Ok(())
    }
}
