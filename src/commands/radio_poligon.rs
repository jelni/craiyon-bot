use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::azuracast;
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities;

pub struct RadioPoligon;

#[async_trait]
impl CommandTrait for RadioPoligon {
    fn command_names(&self) -> &[&str] {
        &["radio_poligon", "poligon"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let station = azuracast::station_now_playing(
            ctx.bot_state.http_client.clone(),
            "radio.poligon.lgbt",
            1,
        )
        .await?;

        ctx.reply_formatted_text(message_entities::formatted_text(station.format_entities(false)))
            .await?;

        Ok(())
    }
}
