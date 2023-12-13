use std::io::BufWriter;

use async_trait::async_trait;
use image::ImageFormat;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;

use super::{CommandResult, CommandTrait};
use crate::apis::craiyon::{self, Model};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{ToEntity, ToEntityOwned};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils::TruncateWithEllipsis;
use crate::utilities::{image_utils, message_entities, telegram_utils, text_utils};

pub struct Generate;

#[async_trait]
impl CommandTrait for Generate {
    fn command_names(&self) -> &[&str] {
        &["generate"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply_formatted_text(message_entities::formatted_text(vec![
            "this command was renamed to ".text(),
            "/craiyon_art".code(),
            ", ".text(),
            "/craiyon_drawing".code(),
            ", ".text(),
            "/craiyon_photo".code(),
            ", and ".text(),
            "/craiyon".code(),
            ". use one of them instead.".text(),
        ]))
        .await?;

        Ok(())
    }
}

pub struct Craiyon {
    command_names: &'static [&'static str],
    description: &'static str,
    model: Model,
}

impl Craiyon {
    pub const fn art() -> Self {
        Self {
            command_names: &["craiyon_art"],
            description: "generate images using 🖍 Craiyon V3 Art model",
            model: Model::Art,
        }
    }

    pub const fn drawing() -> Self {
        Self {
            command_names: &["craiyon_drawing"],
            description: "generate images using 🖍 Craiyon V3 Drawing model",
            model: Model::Drawing,
        }
    }

    pub const fn photo() -> Self {
        Self {
            command_names: &["craiyon_photo"],
            description: "generate images using 🖍 Craiyon V3 Photo model",
            model: Model::Photo,
        }
    }

    pub const fn none() -> Self {
        Self {
            command_names: &["craiyon"],
            description: "generate images using 🖍 Craiyon V3 None model",
            model: Model::None,
        }
    }
}

#[async_trait]
impl CommandTrait for Craiyon {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(2, 30)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            Err(issue)?;
        }

        let truncated_prompt = prompt.clone().truncate_with_ellipsis(256);

        let status_msg = ctx
            .bot_state
            .message_queue
            .wait_for_message(ctx.reply(format!("drawing {truncated_prompt}…")).await?.id)
            .await?;

        let result =
            craiyon::draw(ctx.bot_state.http_client.clone(), self.model, "", &prompt).await?;

        let tasks = result
            .images
            .clone()
            .into_iter()
            .map(|url| {
                let http_client = ctx.bot_state.http_client.clone();
                tokio::spawn(async move {
                    let response = http_client.get(url).send().await;
                    match response {
                        Ok(response) => response.bytes().await,
                        Err(err) => Err(err),
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut images = Vec::with_capacity(tasks.len());
        for task in tasks {
            images.push(task.await.unwrap()?);
        }

        let images = images
            .into_iter()
            .flat_map(|image| image::load_from_memory_with_format(&image, ImageFormat::WebP))
            .collect::<Vec<_>>();

        let image = image_utils::collage(images, (256, 256), 8);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Png).unwrap();

        let download_urls = result
            .images
            .into_iter()
            .enumerate()
            .flat_map(|(i, url)| [" ".text(), (i + 1).to_string().text_url_owned(url)])
            .skip(1);

        let mut entities = vec![
            "drawn ".text(),
            truncated_prompt.bold(),
            " in ".text(),
            text_utils::format_duration(result.duration.as_secs()).text_owned(),
            ".\ndownload: ".text(),
        ];

        entities.extend(download_urls);
        entities.extend([
            "\nsuggested prompt: ".text(),
            result.next_prompt.truncate_with_ellipsis(512).code_owned(),
        ]);

        let message = ctx
            .reply_custom(
                InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                    photo: InputFile::Local(InputFileLocal {
                        path: temp_file.path().to_str().unwrap().into(),
                    }),
                    thumbnail: None,
                    added_sticker_file_ids: Vec::new(),
                    width: image.width().try_into().unwrap(),
                    height: image.height().try_into().unwrap(),
                    caption: Some(message_entities::formatted_text(entities)),
                    self_destruct_type: None,
                    has_spoiler: false,
                }),
                Some(telegram_utils::donate_markup("🖍️ Craiyon", "https://craiyon.com/donate")),
            )
            .await?;

        ctx.bot_state.message_queue.wait_for_message(message.id).await?;
        ctx.delete_message(status_msg.id).await.ok();
        temp_file.close().unwrap();

        Ok(())
    }
}
