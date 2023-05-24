use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::apis::poligon::{self, Song};
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, Entity, ToEntity, ToEntityOwned};
use crate::utilities::text_utils::{self};

pub struct RadioPoligon;

#[async_trait]
impl CommandTrait for RadioPoligon {
    fn command_names(&self) -> &[&str] {
        &["radio_poligon", "radio", "poligon"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let now_playing = poligon::now_playing(ctx.http_client.clone(), 1).await?;
        let mut entities = Vec::new();

        if now_playing.live.is_live {
            entities.extend([
                "DJ ".text(),
                now_playing.live.streamer_name.bold(),
                " NA BEACIE!!!".text(),
                "\n".text(),
            ]);
        } else {
            entities.extend([
                now_playing.station.name.text_url(now_playing.station.public_player_url),
                " mówi:".text(),
                "\n".text(),
            ]);
        }

        if let Some(now_playing) = now_playing.now_playing {
            entities.extend(format_song(now_playing.song));
            entities.extend([
                "\n".text(),
                text_utils::progress_bar(now_playing.elapsed, now_playing.duration).code_owned(),
                " ".text(),
                text_utils::format_duration(now_playing.elapsed.into()).text_owned(),
                " / ".text(),
                text_utils::format_duration(now_playing.duration.into()).text_owned(),
                "\n".text(),
            ]);
        }

        if let Some(playing_next) = now_playing.playing_next {
            entities.push("\nNastępnie:\n".text());
            entities.extend(format_song(playing_next.song));
            entities.push("\n".text());
        }

        entities
            .extend(["\nSłucha: ".text(), now_playing.listeners.total.to_string().text_owned()]);

        if now_playing.listeners.total != now_playing.listeners.unique {
            entities.extend([
                "(tak naprawdę ".text(),
                now_playing.listeners.unique.to_string().text_owned(),
                ")".text(),
            ]);
        }

        ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;

        Ok(())
    }
}

fn format_song(song: Song) -> Vec<Entity<'static>> {
    let mut entities = vec![song.title.bold_owned()];

    if !song.artist.is_empty() {
        entities.extend(["\n".text(), song.artist.text_owned()]);
    }

    if !song.album.is_empty() {
        entities.extend([" • ".text(), song.album.text_owned()]);
    }

    entities
}
