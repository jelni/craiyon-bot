use std::fmt::Write;

use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::poligon;
use crate::utilities::command_context::CommandContext;
use crate::utilities::text_utils::{self, EscapeMarkdown};

pub struct RadioPoligon;

#[async_trait]
impl CommandTrait for RadioPoligon {
    fn command_names(&self) -> &[&str] {
        &["radio_poligon", "radio", "poligon"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let now_playing = poligon::now_playing(ctx.http_client.clone(), 1).await?;
        let mut text = String::new();

        if now_playing.live.is_live {
            writeln!(
                text,
                "DJ *{}* NA BEACIE\\!\\!\\!",
                EscapeMarkdown(&now_playing.live.streamer_name)
            )
            .unwrap();
        } else {
            writeln!(
                text,
                "[{}]({}) mówi:",
                EscapeMarkdown(&now_playing.station.name),
                now_playing.station.public_player_url
            )
            .unwrap();
        }

        if let Some(now_playing) = now_playing.now_playing {
            writeln!(
                text,
                "{}\n`{}` {} / {}",
                now_playing.song,
                EscapeMarkdown(&text_utils::progress_bar(
                    now_playing.elapsed,
                    now_playing.duration,
                )),
                text_utils::format_duration(now_playing.elapsed.into()),
                text_utils::format_duration(now_playing.duration.into())
            )
            .unwrap();
        }

        if let Some(now_playing) = now_playing.playing_next {
            writeln!(text, "\nNastępnie:\n{}", now_playing.song).unwrap();
        }

        write!(text, "\nSłucha: {}", now_playing.listeners.total).unwrap();
        if now_playing.listeners.total != now_playing.listeners.unique {
            write!(text, " \\(tak naprawdę {}\\)", now_playing.listeners.unique).unwrap();
        }

        ctx.reply_markdown(text).await?;

        Ok(())
    }
}
