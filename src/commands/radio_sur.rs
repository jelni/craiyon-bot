use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::azuracast;
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, ToEntity};

pub struct RadioSur;

#[async_trait]
impl CommandTrait for RadioSur {
    fn command_names(&self) -> &[&str] {
        &["radio_sur", "sur"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let mut stations =
            azuracast::now_playing(ctx.bot_state.http_client.clone(), "stream.surradio.live")
                .await?;

        stations.sort_unstable_by_key(|station| station.station.id);

        let entities = stations
            .into_iter()
            .map(|station| station.format_entities(true))
            .collect::<Vec<_>>()
            .join(&"\n\n".text());

        ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;

        Ok(())
    }
}
