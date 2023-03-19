use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;

pub struct Start;

#[async_trait]
impl CommandTrait for Start {
    fn command_names(&self) -> &[&str] {
        &["start"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply_markdown(concat!(
            "use the /generate command to generate images\\.\n",
            "*example:* `/generate crayons in a box`"
        ))
        .await?;

        Ok(())
    }
}
