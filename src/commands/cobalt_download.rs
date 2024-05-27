use std::borrow::Cow;

use async_trait::async_trait;
use rand::seq::IteratorRandom;
use tdlib::enums::{InputFile, InputMessageContent, InputMessageReplyTo, Messages};
use tdlib::functions;
use tdlib::types::{
    InputFileLocal, InputMessageAudio, InputMessageDocument, InputMessageReplyToMessage,
    InputMessageVideo,
};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::cobalt::{self, Error};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::file_download::{DownloadError, NetworkFile};
use crate::utilities::message_entities::{self, ToEntity};
use crate::utilities::telegram_utils;

const MAIN_INSTANCE: &str = "api.cobalt.tools";

pub struct CobaltDownload {
    audio_only: bool,
    command_names: &'static [&'static str],
    description: &'static str,
}

impl CobaltDownload {
    pub const fn auto() -> Self {
        Self {
            audio_only: false,
            command_names: &["cobalt_download", "cobalt", "download", "dl"],
            description: "download online media using ≫ cobalt",
        }
    }

    pub const fn audio() -> Self {
        Self {
            audio_only: true,
            command_names: &["cobalt_download_audio", "cobalt_audio", "download_audio", "dla"],
            description: "download audio using ≫ cobalt",
        }
    }
}

#[async_trait]
impl CommandTrait for CobaltDownload {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(media_url) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let (instance, mut urls) =
            get_urls(ctx.bot_state.http_client.clone(), &media_url, self.audio_only).await?;

        urls.truncate(10);

        let status_msg = ctx
            .bot_state
            .message_queue
            .wait_for_message(
                ctx.reply_formatted_text(message_entities::formatted_text(vec![
                    "downloading from ".text(),
                    instance.as_ref().code(),
                    "…".text(),
                ]))
                .await?
                .id,
            )
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

        Box::pin(send_files(ctx, &files)).await?;

        ctx.delete_message(status_msg.id).await.ok();

        for file in files {
            file.close().unwrap();
        }

        Ok(())
    }
}

async fn get_urls(
    http_client: reqwest::Client,
    url: &str,
    audio_only: bool,
) -> Result<(Cow<str>, Vec<String>), CommandError> {
    match cobalt::query(http_client.clone(), MAIN_INSTANCE, url, audio_only).await {
        Ok(urls) => Ok((Cow::Borrowed(MAIN_INSTANCE), urls)),
        Err(err) => {
            let mut instances = cobalt::instances(http_client.clone()).await?;

            #[allow(clippy::float_cmp)]
            instances.retain(|instance| {
                instance.api_online
                    && instance.score == 100.
                    && instance.protocol == "https"
                    && instance.api != MAIN_INSTANCE
            });

            let instance = instances
                .into_iter()
                .choose(&mut rand::thread_rng())
                .ok_or_else(|| CommandError::Custom("no ≫ cobalt instances online.".into()))?;

            if let Ok(urls) = cobalt::query(http_client, &instance.api, url, audio_only).await {
                Ok((Cow::Owned(instance.api), urls))
            } else {
                Err(match err {
                    Error::Cobalt(err) => err.into(),
                    Error::Server(err) => err.into(),
                    Error::Network(err) => err.into(),
                })
            }
        }
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

async fn send_files(ctx: &CommandContext, files: &[NetworkFile]) -> CommandResult {
    if let [file] = files {
        ctx.bot_state
            .message_queue
            .wait_for_message(
                ctx.reply_custom(
                    get_message_content(file),
                    Some(telegram_utils::donate_markup("≫ cobalt", "https://boosty.to/wukko")),
                )
                .await?
                .id,
            )
            .await?;

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
            &messages.messages.into_iter().flatten().map(|message| message.id).collect::<Vec<_>>(),
        )
        .await
    {
        result?;
    }

    Ok(())
}
