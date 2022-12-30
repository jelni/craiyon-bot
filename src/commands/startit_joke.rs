use std::sync::Arc;

use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::poligon;
use crate::utilities::command_context::CommandContext;

#[derive(Default)]
pub struct StartItJoke;

#[async_trait]
impl CommandTrait for StartItJoke {
    fn command_names(&self) -> &[&str] {
        &["startit_joke", "startit"]
    }

    async fn execute(&self, ctx: Arc<CommandContext>, _: Option<String>) -> CommandResult {
        let joke = poligon::startit_joke(ctx.http_client.clone()).await?;
        ctx.reply(format!("Kacper Podpora mówi: {joke}")).await?;

        Ok(())
    }
}
