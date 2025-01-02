use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{
    InputFileLocal, InputFileRemote, InputMessageAudio, InputMessageDocument, InputMessageVideo,
    InputThumbnail,
};
use tempfile::TempDir;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{Entity, ToEntity, ToEntityOwned, ToNestedEntity};
use crate::utilities::yt_dlp::Infojson;
use crate::utilities::{message_entities, yt_dlp};

const BRUH_EXTRACTORS: [&str; 45] = [
    "alphaporno",
    "beeg",
    "behindkink",
    "bongacams",
    "cam4",
    "camsoda",
    "chaturbate",
    "drtuber",
    "eporner",
    "erocast",
    "eroprofile",
    "goshgay",
    "hotnewhiphop",
    "lovehomeporn",
    "manyvids",
    "motherless",
    "nubilesporn",
    "nuvid",
    "peekvids",
    "pornbox",
    "pornflip",
    "pornhub",
    "pornotube",
    "pornovoisines",
    "pornoxo",
    "redgifs",
    "redtube",
    "rule34video",
    "sexu",
    "slutload",
    "spankbang",
    "stripchat",
    "sunporno",
    "theholetv",
    "thisvid",
    "tnaflix",
    "tube8",
    "txxx",
    "xhamster",
    "xnxx",
    "xvideos",
    "xxxymovies",
    "youjizz",
    "youporn",
    "zenporn",
];

pub struct YtDlp {
    command_names: &'static [&'static str],
    command_description: &'static str,
    format: Option<&'static str>,
}

impl YtDlp {
    pub const fn video() -> Self {
        Self {
            command_names: &["yt_dlp", "ytdlp", "yt"],
            command_description: "Download media using yt-dlp",
            format: None,
        }
    }

    pub const fn audio() -> Self {
        Self {
            command_names: &["yt_dlp_audio", "ytdlp_audio", "yta"],
            command_description: "Download audio using yt-dlp",
            format: Some("bestaudio"),
        }
    }
}

#[async_trait]
impl CommandTrait for YtDlp {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.command_description)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(argument) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let temp_dir = TempDir::new().unwrap();

        let (infojson_path, infojson) =
            yt_dlp::get_infojson(temp_dir.path(), &argument, self.format).await?;

        if BRUH_EXTRACTORS.contains(&infojson.extractor.as_str()) {
            return Err("bruh".into());
        }

        if let Some(live_status) = &infojson.live_status {
            match live_status.as_str() {
                "is_live" => return Err("cannot download livestreams.".into()),
                "is_upcoming" => return Err("this is an upcoming livestream.".into()),
                "post_live" => return Err("this livestream is still processing.".into()),
                _ => (),
            }
        }

        if let Some(duration) = infojson.duration {
            if duration > 60 * 60 {
                return Err("cannot download media longer than 1h.".into());
            }
        } else if infojson.filesize_approx.is_none() {
            return Err("cannot download media, because duration and filesize are unknown.".into());
        }

        if let Some(filesize) = infojson.filesize_approx {
            if filesize > 1024 * 1024 * 1024 {
                return Err("cannot download media larger than 1 GiB.".into());
            }
        }

        let channel = infojson.artist.clone().or_else(|| infojson.channel.clone());
        let title = infojson.track.clone().or_else(|| infojson.title.clone());

        let media_name = [channel.clone(), title.clone()].into_iter().flatten().collect::<Vec<_>>();

        let media_name = if media_name.is_empty() {
            None
        } else {
            Some(media_name.join(" – ").italic_owned().text_url(infojson.webpage_url.clone()))
        };

        let status_message = ctx
            .bot_state
            .message_queue
            .wait_for_message(
                ctx.reply_formatted_text(message_entities::formatted_text(vec![
                    "downloading ".text(),
                    media_name.clone().unwrap_or_else(|| infojson.webpage_url.text()),
                    "…".text(),
                ]))
                .await?
                .id,
            )
            .await?;

        yt_dlp::download_from_infojson(temp_dir.path(), &infojson_path, self.format).await?;

        ctx.bot_state
            .message_queue
            .wait_for_message(
                ctx.reply_custom(
                    get_message_content(&temp_dir, infojson, media_name, channel, title),
                    None,
                )
                .await?
                .id,
            )
            .await?;

        ctx.delete_message(status_message.id).await?;
        temp_dir.close().unwrap();

        Ok(())
    }
}

fn get_message_content(
    temp_dir: &TempDir,
    infojson: Infojson,
    media_name: Option<Entity<'_>>,
    channel: Option<String>,
    title: Option<String>,
) -> InputMessageContent {
    let path = temp_dir.path().join(infojson.filename).to_str().unwrap().into();

    let thumbnail = infojson.thumbnail.map(|thumbnail| InputThumbnail {
        thumbnail: InputFile::Remote(InputFileRemote { id: thumbnail }),
        width: 0,
        height: 0,
    });

    let duration =
        infojson.duration.map(|duration| duration.try_into().unwrap()).unwrap_or_default();

    match infojson.ext.as_str() {
        "mp4" | "mov" | "webm" | "flv" => {
            InputMessageContent::InputMessageVideo(InputMessageVideo {
                video: InputFile::Local(InputFileLocal { path }),
                thumbnail,
                added_sticker_file_ids: Vec::new(),
                duration,
                width: infojson.width.map(|width| width.try_into().unwrap()).unwrap_or_default(),
                height: infojson
                    .height
                    .map(|height| height.try_into().unwrap())
                    .unwrap_or_default(),
                supports_streaming: true,
                caption: media_name
                    .map(|media_name| message_entities::formatted_text(vec![media_name])),
                show_caption_above_media: false,
                self_destruct_type: None,
                has_spoiler: false,
            })
        }
        "m4a" | "aac" | "mp3" | "ogg" | "opus" => {
            InputMessageContent::InputMessageAudio(InputMessageAudio {
                audio: InputFile::Local(InputFileLocal { path }),
                album_cover_thumbnail: thumbnail,
                duration,
                title: title.unwrap_or_default(),
                performer: channel.unwrap_or_default(),
                caption: None,
            })
        }
        _ => InputMessageContent::InputMessageDocument(InputMessageDocument {
            document: InputFile::Local(InputFileLocal { path }),
            thumbnail,
            disable_content_type_detection: true,
            caption: media_name
                .map(|media_name| message_entities::formatted_text(vec![media_name])),
        }),
    }
}
