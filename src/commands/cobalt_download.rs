use std::borrow::Cow;
use std::env;
use std::path::Path;

use async_trait::async_trait;
use serde::Deserialize;
use tdlib::enums::{InputFile, InputMessageContent, InputMessageReplyTo, Messages};
use tdlib::functions;
use tdlib::types::{
    InputFileLocal, InputFileRemote, InputMessageAudio, InputMessageDocument,
    InputMessageReplyToMessage, InputMessageVideo, InputThumbnail,
};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::cobalt::{self, Error, Response};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::file_download::NetworkFile;
use crate::utilities::message_entities::{self, ToEntity};
use crate::utilities::{ffprobe, telegram_utils};

const TWITTER_REPLACEMENTS: [&str; 7] = [
    "fxtwitter.com",
    "fixupx.com",
    "twittpr.com",
    "vxtwitter.com",
    "fixvx.com",
    "girlcockx.com",
    "stupidpenisx.com",
];

#[derive(Deserialize)]
struct CobaltInstance<'a> {
    name: &'a str,
    url: &'a str,
    api_key: Option<&'a str>,
}

pub struct CobaltDownload {
    command_names: &'static [&'static str],
    description: &'static str,
    audio_only: bool,
}

impl CobaltDownload {
    pub const fn auto() -> Self {
        Self {
            command_names: &["cobalt_download", "cobalt", "download", "dl"],
            description: "download online media using ≫ cobalt",
            audio_only: false,
        }
    }

    pub const fn audio() -> Self {
        Self {
            command_names: &["cobalt_download_audio", "cobalt_audio", "download_audio", "dla"],
            description: "download audio using ≫ cobalt",
            audio_only: true,
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

        let (instance, result) =
            match get_result(&ctx.bot_state.http_client, &media_url, self.audio_only).await {
                Ok(result) => result,
                Err(err) => {
                    return match err {
                        Error::Server(error) => Err(error.into()),
                        Error::Network(error) => Err(error.into()),
                    };
                }
            };

        Box::pin(send_files(ctx, &instance, result)).await?;

        Ok(())
    }
}

async fn get_result(
    http_client: &reqwest::Client,
    url: &str,
    audio_only: bool,
) -> Result<(String, Response), cobalt::Error> {
    let url = TWITTER_REPLACEMENTS
        .into_iter()
        .find_map(|replacement| url.strip_prefix(&format!("https://{replacement}/")))
        .map(|path| Cow::Owned(format!("https://twitter.com/{path}")))
        .unwrap_or(Cow::Borrowed(url));

    let instances = env::var("COBALT_INSTANCES").unwrap();
    let instances = serde_json::from_str::<Vec<CobaltInstance>>(&instances).unwrap();

    let mut error = None;

    for instance in instances {
        match cobalt::query(http_client, instance.url, instance.api_key, &url, audio_only).await {
            Ok(result) => return Ok((instance.name.into(), result)),
            Err(err) => {
                error = Some(err);
            }
        }
    }

    Err(error.unwrap())
}

#[expect(clippy::too_many_lines, clippy::cognitive_complexity, clippy::large_stack_frames)]
async fn send_files(ctx: &CommandContext, instance: &str, result: Response) -> CommandResult {
    match result {
        Response::Redirect(file) | Response::Tunnel(file) => {
            let status_msg = ctx
                .bot_state
                .message_queue
                .wait_for_message(
                    ctx.reply_formatted_text(message_entities::formatted_text(vec![
                        "downloading from ".text(),
                        instance.code(),
                        "…".text(),
                    ]))
                    .await?
                    .id,
                )
                .await?;

            let network_file = NetworkFile::download(
                &ctx.bot_state.http_client,
                &file.url,
                Some(file.filename.clone()),
                ctx.client_id,
            )
            .await?;

            ctx.edit_message(status_msg.id, "uploading…".into()).await?;

            ctx.bot_state
                .message_queue
                .wait_for_message(
                    ctx.reply_custom(
                        get_message_content(&file.filename, &network_file).await?,
                        Some(telegram_utils::donate_markup(
                            "≫ cobalt",
                            "https://cobalt.tools/donate",
                        )),
                    )
                    .await?
                    .id,
                )
                .await?;

            ctx.delete_message(status_msg.id).await.ok();

            network_file.close().unwrap();
        }
        Response::Picker(picker) => {
            let status_msg = ctx
                .bot_state
                .message_queue
                .wait_for_message(
                    ctx.reply_formatted_text(message_entities::formatted_text(vec![
                        "downloading from ".text(),
                        instance.code(),
                        "…".text(),
                    ]))
                    .await?
                    .id,
                )
                .await?;

            let mut picker_items = picker.picker;
            picker_items.truncate(10);
            let mut files = Vec::with_capacity(picker_items.len());

            for item in &picker_items {
                files.push(
                    NetworkFile::download(
                        &ctx.bot_state.http_client,
                        &item.url,
                        None,
                        ctx.client_id,
                    )
                    .await?,
                );
            }

            let audio_file = if let Some(url) = picker.audio {
                Some(
                    NetworkFile::download(
                        &ctx.bot_state.http_client,
                        &url,
                        picker.audio_filename,
                        ctx.client_id,
                    )
                    .await?,
                )
            } else {
                None
            };

            ctx.edit_message(status_msg.id, "uploading…".into()).await?;

            let messages = files
                .iter()
                .zip(picker_items)
                .map(|(file, item)| {
                    InputMessageContent::InputMessageDocument(InputMessageDocument {
                        document: InputFile::Local(InputFileLocal {
                            path: file.file_path.to_str().unwrap().into(),
                        }),
                        thumbnail: item.thumb.map(|thumbnail| InputThumbnail {
                            thumbnail: InputFile::Remote(InputFileRemote { id: thumbnail }),
                            width: 0,
                            height: 0,
                        }),
                        disable_content_type_detection: true,
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

            for file in files {
                file.close().unwrap();
            }

            if let Some(audio_file) = audio_file {
                ctx.bot_state
                    .message_queue
                    .wait_for_message(
                        ctx.reply_custom(
                            InputMessageContent::InputMessageAudio(InputMessageAudio {
                                audio: InputFile::Local(InputFileLocal {
                                    path: audio_file.file_path.to_str().unwrap().into(),
                                }),
                                album_cover_thumbnail: None,
                                duration: 0,
                                title: String::new(),
                                performer: String::new(),
                                caption: None,
                            }),
                            None,
                        )
                        .await?
                        .id,
                    )
                    .await?;

                audio_file.close().unwrap();
            }

            ctx.delete_message(status_msg.id).await.ok();
        }
        Response::Error(error) => {
            let text = match cobalt::get_error_localization(&ctx.bot_state.http_client).await {
                Ok(mut localization) => localization
                    .remove(&error.error.code["error.".len()..])
                    .unwrap_or(error.error.code),
                Err(err) => {
                    log::warn!("failed to get cobalt localization: {err}");
                    error.error.code
                }
            };

            return Err(text.into());
        }
    }

    Ok(())
}

async fn get_message_content(
    filename: &str,
    file: &NetworkFile,
) -> Result<InputMessageContent, CommandError> {
    let input_file =
        InputFile::Local(InputFileLocal { path: file.file_path.to_str().unwrap().into() });

    if let Some(file_extension) = Path::new(filename).extension() {
        if file_extension.eq_ignore_ascii_case("mp4") {
            let ffprobe = ffprobe::ffprobe(&file.file_path).await?;

            let video_stream = ffprobe.streams.and_then(|streams| {
                streams.into_iter().find(|stream| {
                    stream.codec_type.as_ref().is_some_and(|codec_type| codec_type == "video")
                })
            });

            return Ok(InputMessageContent::InputMessageVideo(InputMessageVideo {
                video: input_file,
                thumbnail: None,
                cover: None,
                start_timestamp: 0,
                added_sticker_file_ids: Vec::new(),
                #[expect(clippy::cast_possible_truncation)]
                duration: ffprobe
                    .format
                    .map(|format| format.duration.parse::<f32>().unwrap() as i32)
                    .unwrap_or_default(),
                width: video_stream
                    .as_ref()
                    .and_then(|stream| stream.width.map(|width| width.try_into().unwrap()))
                    .unwrap_or_default(),
                height: video_stream
                    .and_then(|stream| stream.height.map(|height| height.try_into().unwrap()))
                    .unwrap_or_default(),
                supports_streaming: true,
                caption: None,
                show_caption_above_media: false,
                self_destruct_type: None,
                has_spoiler: false,
            }));
        } else if ["mp3", "opus", "weba"]
            .into_iter()
            .any(|extension| file_extension.eq_ignore_ascii_case(extension))
        {
            let ffprobe = ffprobe::ffprobe(&file.file_path).await?;

            let audio_stream = ffprobe.streams.and_then(|streams| {
                streams.into_iter().find(|stream| {
                    stream.codec_type.as_ref().is_some_and(|codec_type| codec_type == "audio")
                })
            });

            let tags = audio_stream.and_then(|stream| stream.tags);

            return Ok(InputMessageContent::InputMessageAudio(InputMessageAudio {
                audio: input_file,
                album_cover_thumbnail: None,
                #[expect(clippy::cast_possible_truncation)]
                duration: ffprobe
                    .format
                    .map(|format| format.duration.parse::<f32>().unwrap() as i32)
                    .unwrap_or_default(),
                title: tags.as_ref().and_then(|tags| tags.title.clone()).unwrap_or_default(),
                performer: tags.and_then(|tags| tags.artist).unwrap_or_default(),
                caption: None,
            }));
        }
    }

    Ok(InputMessageContent::InputMessageDocument(InputMessageDocument {
        document: input_file,
        thumbnail: None,
        disable_content_type_detection: true,
        caption: None,
    }))
}
