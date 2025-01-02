use std::path::Path;

use serde::Deserialize;
use tokio::process::Command;

#[derive(Deserialize)]
pub struct Ffprobe {
    pub streams: Option<Vec<Streams>>,
    pub format: Option<Format>,
}

#[derive(Deserialize)]
pub struct Streams {
    pub codec_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tags: Option<Tags>,
}

#[derive(Deserialize)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
}

#[derive(Deserialize)]
pub struct Format {
    pub duration: String,
}

pub async fn ffprobe(path: &Path) -> serde_json::Result<Ffprobe> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-output_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path)
        .output()
        .await
        .unwrap();

    serde_json::from_slice(&output.stdout)
}
