use std::sync::Arc;

use async_trait::async_trait;
use image::imageops::FilterType;
use image::ImageFormat;
use tdlib::enums::{FormattedText, InputFile, InputMessageContent, TextParseMode};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessagePhoto, TextParseModeMarkdown};
use tempfile::NamedTempFile;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::craiyon;
use crate::ratelimit::RateLimiter;
use crate::utils::{
    check_prompt, donate_markup, escape_markdown, format_duration, image_collage, Context,
};

#[derive(Default)]
pub struct Generate;

#[async_trait]
impl CommandTrait for Generate {
    fn command_names(&self) -> &[&str] {
        &["generate", "g", "gen", "craiyon"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("generate images using 🖍 Craiyon")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60)
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let prompt = arguments.ok_or(MissingArgument("prompt to generate"))?;

        if let Some(issue) = check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            Err(issue)?;
        }

        let status_msg = ctx
            .message_queue
            .wait_for_message(ctx.reply(format!("generating {prompt}…")).await?.id)
            .await?;
        let result = craiyon::generate(ctx.http_client.clone(), &prompt).await?;

        let images = result
            .images
            .into_iter()
            .flat_map(|image| image::load_from_memory_with_format(&image, ImageFormat::WebP))
            .map(|image| image.resize_exact(256, 256, FilterType::Lanczos3))
            .collect::<Vec<_>>();

        let image = image_collage(images, (256, 256), 3, 8);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(temp_file.as_file_mut(), ImageFormat::Png).unwrap();

        let FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            format!(
                "generated *{}* in {}\\.",
                escape_markdown(prompt),
                format_duration(result.duration.as_secs())
            ),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            ctx.client_id,
        )
        .await
        .unwrap();

        ctx.message_queue
            .wait_for_message(
                ctx.reply_custom(
                    InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                        photo: InputFile::Local(InputFileLocal {
                            path: temp_file.path().to_str().unwrap().into(),
                        }),
                        thumbnail: None,
                        added_sticker_file_ids: Vec::new(),
                        width: image.width().try_into().unwrap(),
                        height: image.height().try_into().unwrap(),
                        caption: Some(formatted_text),
                        ttl: 0,
                    }),
                    Some(donate_markup("🖍️ Craiyon", "https://craiyon.com/donate")),
                )
                .await?
                .id,
            )
            .await?;

        ctx.delete_message(status_msg.id).await.ok();
        temp_file.close().unwrap();

        Ok(())
    }
}
