use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::urbansharing;
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, ToEntity};
use crate::utilities::text_utils;

pub struct Mevo;

#[async_trait]
impl CommandTrait for Mevo {
    fn command_names(&self) -> &[&str] {
        &["mevo"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let stats =
            urbansharing::system_stats(ctx.bot_state.http_client.clone(), "inurba-gdansk").await?;

        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        ctx.reply_formatted_text(message_entities::formatted_text(vec![
            "jadący teraz: ".text(),
            stats.system_active_trip_count.count.to_string().bold(),
            ", dzisiaj: ".text(),
            stats.system_stats.trips_today.to_string().bold(),
            " (".text(),
            stats.system_stats.unique_users_today.to_string().bold(),
            " użytkowników), wczoraj: ".text(),
            stats.system_stats.trips_yesterday.to_string().bold(),
            "\nśredni czas dzisiaj: ".text(),
            text_utils::format_duration(stats.system_stats.median_duration_today as u64).bold(),
            ", w tym roku: ".text(),
            text_utils::format_duration(stats.system_stats.median_duration_this_year as u64).bold(),
        ]))
        .await?;

        Ok(())
    }
}
