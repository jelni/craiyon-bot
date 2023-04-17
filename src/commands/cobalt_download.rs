use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent, Messages};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessageDocument};

use super::{CommandResult, CommandTrait};
use crate::apis::cobalt;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::file_download::{DownloadError, NetworkFile};
use crate::utilities::telegram_utils;

pub struct CobaltDownload;

#[async_trait]
impl CommandTrait for CobaltDownload {
    fn command_names(&self) -> &[&str] {
        &["cobalt_download", "cobalt", "download", "dl"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("download online media using ≫ cobalt")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(media_url) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;
        let urls = cobalt::query(ctx.http_client.clone(), &media_url)
            .await??
            .into_iter()
            .take(10)
            .collect::<Vec<_>>();

        let status_msg =
            ctx.message_queue.wait_for_message(ctx.reply("downloading…").await?.id).await?;

        let mut files = Vec::with_capacity(urls.len());

        for url in urls {
            match NetworkFile::download(ctx.http_client.clone(), &url, ctx.client_id).await {
                Ok(file) => files.push(file),
                Err(err) => match err {
                    DownloadError::RequestError(err) => {
                        log::warn!("cobalt download failed: {err}");
                        Err("≫ cobalt download failed.")?;
                    }
                    DownloadError::FilesystemError => {
                        Err("failed to save the file to the hard drive.")?;
                    }
                    DownloadError::InvalidResponse => {
                        Err("got invalid HTTP response while downloading the file")?;
                    }
                },
            }
        }

        ctx.edit_message(status_msg.id, "uploading…").await?;

        if files.len() == 1 {
            let file = files.into_iter().next().unwrap();

            ctx.message_queue
                .wait_for_message(
                    ctx.reply_custom(
                        InputMessageContent::InputMessageDocument(InputMessageDocument {
                            document: InputFile::Local(InputFileLocal {
                                path: file.file_path.clone(),
                            }),
                            thumbnail: None,
                            disable_content_type_detection: false,
                            caption: None,
                        }),
                        Some(telegram_utils::donate_markup("≫ cobalt", "https://boosty.to/wukko")),
                    )
                    .await?
                    .id,
                )
                .await?;

            ctx.delete_message(status_msg.id).await.ok();
            file.close().unwrap();

            return Ok(());
        }

        let messages = files
            .iter()
            .map(|file| {
                InputMessageContent::InputMessageDocument(InputMessageDocument {
                    document: InputFile::Local(InputFileLocal { path: file.file_path.clone() }),
                    thumbnail: None,
                    disable_content_type_detection: false,
                    caption: None,
                })
            })
            .collect::<Vec<_>>();

        let Messages::Messages(messages) = functions::send_message_album(
            ctx.message.chat_id,
            ctx.message.message_thread_id,
            ctx.message.id,
            None,
            messages,
            false,
            ctx.client_id,
        )
        .await?;

        for result in ctx
            .message_queue
            .wait_for_messages(messages.messages.into_iter().flatten().map(|message| message.id))
            .await
        {
            result?;
        }

        ctx.delete_message(status_msg.id).await.ok();

        for file in files {
            file.close().unwrap();
        }

        Ok(())
    }
}
