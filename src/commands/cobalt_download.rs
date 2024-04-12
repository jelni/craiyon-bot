use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent, InputMessageReplyTo, Messages};
use tdlib::functions;
use tdlib::types::{
    InputFileLocal, InputMessageAudio, InputMessageDocument, InputMessageVideo,
    InputMessageReplyToMessage,
};

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
        let urls = cobalt::query(ctx.bot_state.http_client.clone(), &media_url)
            .await??
            .into_iter()
            .take(10)
            .collect::<Vec<_>>();

        let status_msg = ctx
            .bot_state
            .message_queue
            .wait_for_message(ctx.reply("downloading…".into()).await?.id)
            .await?;

        let mut files = Vec::with_capacity(urls.len());

        for url in urls {
            match NetworkFile::download(ctx.bot_state.http_client.clone(), &url, ctx.client_id)
                .await
            {
                Ok(file) => files.push(file),
                Err(err) => match err {
                    DownloadError::RequestError(err) => {
                        log::warn!("cobalt download failed: {err}");
                        Err(format!("≫ cobalt download failed: {}", err.without_url()))?;
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

        ctx.edit_message(status_msg.id, "uploading…".into()).await?;

        if files.len() == 1 {
            let file = files.into_iter().next().unwrap();

            ctx.bot_state
                .message_queue
                .wait_for_message(
                    ctx.reply_custom(
                        get_message_content(&file),
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
                    document: InputFile::Local(InputFileLocal {
                        path: file.file_path.to_str().unwrap().into(),
                    }),
                    thumbnail: None,
                    disable_content_type_detection: false,
                    caption: None,
                })
            })
            .collect::<Vec<_>>();

        let Messages::Messages(messages) = functions::send_message_album(
            ctx.message.chat_id,
            ctx.message.message_thread_id,
            Some(InputMessageReplyTo::Message(InputMessageReplyToMessage {
                chat_id: ctx.message.chat_id,
                message_id: ctx.message.id,
                ..Default::default()
            })),
            None,
            messages,
            ctx.client_id,
        )
        .await?;

        for result in ctx
            .bot_state
            .message_queue
            .wait_for_messages(
                &messages
                    .messages
                    .into_iter()
                    .flatten()
                    .map(|message| message.id)
                    .collect::<Vec<_>>(),
            )
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

fn get_message_content(file: &NetworkFile) -> InputMessageContent {
    let input_file =
        InputFile::Local(InputFileLocal { path: file.file_path.to_str().unwrap().into() });

    if file.content_type.as_ref().is_some_and(|content_type| content_type == "video/mp4")
        || file.file_path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("mp4"))
    {
        InputMessageContent::InputMessageVideo(InputMessageVideo {
            video: input_file,
            thumbnail: None,
            added_sticker_file_ids: Vec::new(),
            duration: 0,
            width: 0,
            height: 0,
            supports_streaming: true,
            caption: None,
            self_destruct_type: None,
            has_spoiler: false,
        })
    } else if file
        .content_type
        .as_ref()
        .is_some_and(|content_type| ["audio/mpeg", "audio/webm"].contains(&content_type.as_str()))
        || file.file_path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("mp3") || extension.eq_ignore_ascii_case("opus")
        })
    {
        InputMessageContent::InputMessageAudio(InputMessageAudio {
            audio: input_file,
            album_cover_thumbnail: None,
            duration: 0,
            title: String::new(),
            performer: String::new(),
            caption: None,
        })
    } else {
        InputMessageContent::InputMessageDocument(InputMessageDocument {
            document: input_file,
            thumbnail: None,
            disable_content_type_detection: false,
            caption: None,
        })
    }
}
