use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug)]
pub enum Error {
    YtDlp(String),
    Serde(serde_json::Error),
}

#[derive(Deserialize)]
pub struct Infojson {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub duration: Option<u32>,
    pub webpage_url: String,
    pub live_status: Option<String>,
    pub track: Option<String>,
    pub channel: Option<String>,
    pub extractor: String,
    pub artist: Option<String>,
    pub ext: String,
    pub filesize_approx: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub filename: String,
}

pub async fn get_infojson(
    working_directory: &Path,
    query: &str,
    format: Option<&str>,
) -> Result<(PathBuf, Infojson), Error> {
    let mut command = Command::new("yt-dlp");

    command
        .current_dir(working_directory)
        .arg("--default-search")
        .arg("auto")
        .arg("--output")
        .arg("infojson:_")
        .arg("--write-info-json")
        .arg("--no-clean-info-json")
        .arg("--skip-download");

    if let Some(format) = format {
        command.arg("--format").arg(format);
    }

    let output = command.arg("--format-sort").arg("ext").arg(query).output().await.unwrap();

    if !output.status.success() {
        return Err(Error::YtDlp(String::from_utf8(output.stderr).unwrap()));
    }

    let infojson_path = working_directory.join("_.info.json");
    let infojson = serde_json::from_reader(BufReader::new(File::open(&infojson_path).unwrap()))
        .map_err(Error::Serde)?;

    Ok((infojson_path, infojson))
}

pub async fn download_from_infojson(
    working_directory: &Path,
    infojson_path: &Path,
    format: Option<&str>,
) -> Result<(), String> {
    let mut command = Command::new("yt-dlp");
    command.current_dir(working_directory).arg("--load-info-json").arg(infojson_path);

    if let Some(format) = format {
        command.arg("--format").arg(format);
    }

    let output = command.arg("--format-sort").arg("ext").output().await.unwrap();

    if !output.status.success() {
        return Err(String::from_utf8(output.stderr).unwrap());
    }

    Ok(())
}
