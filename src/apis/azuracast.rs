use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;
use crate::utilities::message_entities::{Entity, ToEntity, ToEntityOwned};
use crate::utilities::text_utils;

#[derive(Deserialize)]
pub struct PlayerState {
    pub station: Station,
    pub live: Live,
    pub now_playing: Option<CurrentSong>,
    pub playing_next: Option<StationQueue>,
    pub listeners: Listeners,
}

#[derive(Deserialize)]
pub struct Station {
    pub id: u32,
    pub name: String,
    pub public_player_url: String,
}

#[derive(Deserialize)]
pub struct Listeners {
    pub total: u32,
    pub unique: u32,
}

#[derive(Deserialize)]
pub struct Live {
    pub is_live: bool,
    pub streamer_name: String,
}

#[derive(Deserialize)]
pub struct CurrentSong {
    pub song: Song,
    pub duration: u32,
    pub elapsed: u32,
}

#[derive(Deserialize)]
pub struct StationQueue {
    pub song: Song,
}

#[derive(Deserialize)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
}

impl PlayerState {
    pub fn format_entities(self, compact: bool) -> Vec<Entity<'static>> {
        let mut entities = Vec::new();

        if self.live.is_live {
            entities.extend([
                "DJ ".text(),
                self.live.streamer_name.bold_owned(),
                " NA BEACIE!!!".text(),
            ]);
        } else {
            entities.extend([
                self.station.name.text_url_owned(self.station.public_player_url),
                " mówi:".text(),
            ]);
        }

        if let Some(now_playing) = self.now_playing {
            entities.push("\n".text());
            entities.extend(format_song(now_playing.song));
            entities.extend([
                "\n".text(),
                text_utils::progress_bar(now_playing.elapsed, now_playing.duration).code_owned(),
                " ".text(),
                text_utils::format_duration(now_playing.elapsed.into()).text_owned(),
                " / ".text(),
                text_utils::format_duration(now_playing.duration.into()).text_owned(),
            ]);
        }

        if !compact && let Some(playing_next) = self.playing_next {
            entities.push("\n\nNastępnie:\n".text());
            entities.extend(format_song(playing_next.song));
        }

        if !compact || self.listeners.total > 0 {
            if !compact {
                entities.push("\n".text());
            }

            entities.extend(["\nSłucha: ".text(), self.listeners.total.to_string().text_owned()]);

            if self.listeners.total != self.listeners.unique {
                entities.extend([
                    "(tak naprawdę ".text(),
                    self.listeners.unique.to_string().text_owned(),
                    ")".text(),
                ]);
            }
        }

        entities
    }
}

pub async fn now_playing(
    http_client: reqwest::Client,
    domain: &str,
) -> Result<Vec<PlayerState>, CommandError> {
    let now_playing = http_client
        .get(format!("https://{domain}/api/nowplaying"))
        .send()
        .await?
        .server_error()?
        .json()
        .await?;

    Ok(now_playing)
}

pub async fn station_now_playing(
    http_client: reqwest::Client,
    domain: &str,
    station_id: u32,
) -> Result<PlayerState, CommandError> {
    let now_playing = http_client
        .get(format!("https://{domain}/api/nowplaying/{station_id}"))
        .send()
        .await?
        .server_error()?
        .json()
        .await?;

    Ok(now_playing)
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
