use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, ToEntity};

pub struct Start;

#[async_trait]
impl CommandTrait for Start {
    fn command_names(&self) -> &[&str] {
        &["start"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply_formatted_text(message_entities::formatted_text(vec![
            "use the /craiyon_art command to generate images.\n".text(),
            "example:".bold(),
            " ".text(),
            "/craiyon_art crayons in a box".code(),
        ]))
        .await?;

        Ok(())
    }
}
