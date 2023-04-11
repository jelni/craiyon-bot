use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::clippy;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};

pub struct Clippy;

#[async_trait]
impl CommandTrait for Clippy {
    fn command_names(&self) -> &[&str] {
        &["hopbot", "clippy"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask HopBot a question")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(question) = ConvertArgument::convert(ctx, &arguments).await?.0;

        let text = clippy::query(ctx.http_client.clone(), &question).await?;

        ctx.reply(text).await?;

        Ok(())
    }
}
