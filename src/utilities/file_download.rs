use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::CONTENT_DISPOSITION;
use tdlib::{enums, functions};
use tempfile::TempDir;

pub const MEBIBYTE: i64 = 1024 * 1024;

#[derive(Debug)]
pub enum DownloadError {
    RequestError(reqwest::Error),
    FilesystemError,
}

pub struct NetworkFile {
    temp_dir: TempDir,
    pub file_path: PathBuf,
}

impl NetworkFile {
    pub async fn download(
        http_client: &reqwest::Client,
        url: &str,
        filename: Option<String>,
        client_id: i32,
    ) -> Result<Self, DownloadError> {
        let response = http_client
            .get(url)
            .timeout(Duration::from_secs(3600))
            .send()
            .await
            .map_err(DownloadError::RequestError)?
            .error_for_status()
            .map_err(DownloadError::RequestError)?;

        let enums::Text::Text(filename) = functions::clean_file_name(
            filename.unwrap_or_else(|| get_filename(&response).to_string()),
            client_id,
        )
        .await
        .unwrap();

        let temp_dir = TempDir::new().map_err(|_| DownloadError::FilesystemError)?;
        let file_path = temp_dir.path().join(filename.text);

        let mut file = BufWriter::with_capacity(
            4 * 1024 * 1024,
            File::create(&file_path).map_err(|_| DownloadError::FilesystemError)?,
        );

        let mut stream = response.bytes_stream();

        while let Some(bytes) = stream.next().await {
            let bytes = bytes.map_err(DownloadError::RequestError)?;
            file.write_all(&bytes).map_err(|_| DownloadError::FilesystemError)?;
        }

        file.flush().map_err(|_| DownloadError::FilesystemError)?;

        Ok(Self { temp_dir, file_path })
    }

    pub fn close(self) -> io::Result<()> {
        self.temp_dir.close()
    }
}

fn get_filename(response: &reqwest::Response) -> &str {
    response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| parse_filename(header.to_str().unwrap()))
        .unwrap_or_else(|| response.url().path_segments().unwrap().next_back().unwrap())
}

/// parses the `filename` from a `Content-Disposition` header
fn parse_filename(value: &str) -> Option<&str> {
    value.split(';').find_map(|dir| {
        let mut pair = dir.trim().split('=');
        if pair.next().unwrap() == "filename" {
            Some(pair.next().unwrap().trim_matches('"'))
        } else {
            None
        }
    })
}
