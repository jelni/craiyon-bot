use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::poligon;
use crate::utilities::command_context::CommandContext;

pub struct StartitJoke;

#[async_trait]
impl CommandTrait for StartitJoke {
    fn command_names(&self) -> &[&str] {
        &["startit_joke", "startit"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let joke = poligon::startit_joke(ctx.http_client.clone()).await?;
        ctx.reply(format!("Kacper Podpora mówi: {joke}")).await?;

        Ok(())
    }
}
