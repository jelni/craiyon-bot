use std::io::BufWriter;

use async_trait::async_trait;
use image::ImageFormat;
use tdlib::enums::{FormattedText, InputFile, InputMessageContent, TextParseMode};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessagePhoto, TextParseModeMarkdown};
use tempfile::NamedTempFile;

use super::{CommandResult, CommandTrait};
use crate::apis::craiyon::{self, Model};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils::EscapeMarkdown;
use crate::utilities::{image_utils, telegram_utils, text_utils};

pub struct Generate;

#[async_trait]
impl CommandTrait for Generate {
    fn command_names(&self) -> &[&str] {
        &["generate"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply_markdown(concat!(
            "this command was renamed to `/craiyon_art`, `/craiyon_drawing`, ",
            "`/craiyon_photo`, and `/craiyon`\\. use one of them instead\\."
        ))
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
    pub fn art() -> Self {
        Self {
            command_names: &["craiyon_art"],
            description: "generate images using 🖍 Craiyon V3 Art model",
            model: Model::Art,
        }
    }

    pub fn drawing() -> Self {
        Self {
            command_names: &["craiyon_drawing"],
            description: "generate images using 🖍 Craiyon V3 Drawing model",
            model: Model::Drawing,
        }
    }

    pub fn photo() -> Self {
        Self {
            command_names: &["craiyon_photo"],
            description: "generate images using 🖍 Craiyon V3 Photo model",
            model: Model::Photo,
        }
    }

    pub fn none() -> Self {
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

        let status_msg = ctx
            .message_queue
            .wait_for_message(ctx.reply(format!("drawing {prompt}…")).await?.id)
            .await?;

        let result = craiyon::draw(ctx.http_client.clone(), self.model, "", &prompt).await?;

        let tasks = result
            .images
            .clone()
            .into_iter()
            .map(|url| {
                let http_client = ctx.http_client.clone();
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
            .map(|(i, url)| format!("[{}]({url})", i + 1))
            .collect::<Vec<_>>();

        let FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            format!(
                "drawn *{}* in {}\\.\ndownload: {}\nsuggested prompt: `{}`",
                EscapeMarkdown(&prompt),
                text_utils::format_duration(result.duration.as_secs()),
                download_urls.join(" "),
                EscapeMarkdown(&result.next_prompt)
            ),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            ctx.client_id,
        )
        .await
        .unwrap();

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
                    caption: Some(formatted_text),
                    self_destruct_time: 0,
                    has_spoiler: false,
                }),
                Some(telegram_utils::donate_markup("🖍️ Craiyon", "https://craiyon.com/donate")),
            )
            .await?;

        ctx.message_queue.wait_for_message(message.id).await?;
        ctx.delete_message(status_msg.id).await.ok();
        temp_file.close().unwrap();

        Ok(())
    }
}
