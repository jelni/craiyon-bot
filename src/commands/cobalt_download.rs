use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tdlib::enums::{InputFile, InputMessageContent, Text};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessageDocument};
use tempfile::TempDir;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::cobalt;
use crate::command_context::CommandContext;
use crate::utils::donate_markup;

#[derive(Default)]
pub struct CobaltDownload;

#[async_trait]
impl CommandTrait for CobaltDownload {
    fn command_names(&self) -> &[&str] {
        &["cobalt_download", "cobalt", "download", "dl"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("download online media using ≫ cobalt")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let media_url = arguments.ok_or(MissingArgument("URL to download"))?;

        ctx.send_typing().await?;
        let mut urls = cobalt::query(ctx.http_client.clone(), media_url.clone()).await??;

        let status_msg =
            ctx.message_queue.wait_for_message(ctx.reply("downloading…").await?.id).await?;

        urls.truncate(4);
        let mut downloads = Vec::with_capacity(urls.len());

        for url in urls {
            match cobalt::download(ctx.http_client.clone(), url).await {
                Ok(download) if download.media.is_empty() => {
                    Err("≫ cobalt failed to download media. try again later.")?;
                }
                Ok(download) => downloads.push(download),
                Err(err) => {
                    Err(err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string())?;
                }
            }
        }

        let temp_dir = TempDir::new().unwrap();

        for download in downloads {
            let Text::Text(file_name) =
                functions::clean_file_name(download.filename, ctx.client_id).await?;
            let path = temp_dir.path().join(file_name.text);
            let mut file = OpenOptions::new().write(true).create(true).open(&path).unwrap();
            file.write_all(&download.media).unwrap();

            let message = ctx
                .reply_custom(
                    InputMessageContent::InputMessageDocument(InputMessageDocument {
                        document: InputFile::Local(InputFileLocal {
                            path: path.to_str().unwrap().into(),
                        }),
                        thumbnail: None,
                        disable_content_type_detection: false,
                        caption: None,
                    }),
                    Some(donate_markup("≫ cobalt", "https://boosty.to/wukko")),
                )
                .await?;

            ctx.message_queue.wait_for_message(message.id).await?;
        }

        ctx.delete_message(status_msg.id).await.ok();
        temp_dir.close().unwrap();

        Ok(())
    }
}
