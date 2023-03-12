use std::fmt;

use serde::Deserialize;

use crate::commands::CommandError;
use crate::utilities::api_utils::DetectServerError;
use crate::utilities::text_utils::EscapeMarkdown;

#[derive(Deserialize)]
struct StartitJoke {
    joke: String,
}

pub async fn startit_joke(http_client: reqwest::Client) -> Result<String, CommandError> {
    let joke = http_client
        .get("https://astolfo.poligon.lgbt/api/startit")
        .send()
        .await?
        .server_error()?
        .json::<StartitJoke>()
        .await?;

    Ok(joke.joke)
}

#[derive(Deserialize)]
pub struct NowPlaying {
    pub station: Station,
    pub live: Live,
    pub now_playing: Option<CurrentSong>,
    pub playing_next: Option<StationQueue>,
    pub listeners: Listeners,
}

#[derive(Deserialize)]
pub struct Station {
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

pub async fn now_playing(
    http_client: reqwest::Client,
    station_id: u32,
) -> Result<NowPlaying, CommandError> {
    let now_playing = http_client
        .get(format!("https://radio.poligon.lgbt/api/nowplaying/{station_id}"))
        .send()
        .await?
        .server_error()?
        .json::<NowPlaying>()
        .await?;

    Ok(now_playing)
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "*{}*\n{}", EscapeMarkdown(&self.title), EscapeMarkdown(&self.artist))?;

        if !self.album.is_empty() {
            write!(f, " • {}", EscapeMarkdown(&self.album))?;
        }

        Ok(())
    }
}
