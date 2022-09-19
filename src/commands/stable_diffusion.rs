use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use image::{ImageFormat, ImageOutputFormat};
use tgbotapi::requests::{ParseMode, ReplyMarkup};
use tgbotapi::{FileType, InlineKeyboardButton, InlineKeyboardMarkup};

use super::{check_prompt, Command};
use crate::api_methods::SendPhoto;
use crate::apis::stablehorde;
use crate::utils::{escape_markdown, format_duration, image_collage, Context};

pub struct StableDiffusion;

#[async_trait]
impl Command for StableDiffusion {
    fn name(&self) -> &str {
        "stable_diffusion"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prompt = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("prompt to generate").await;
            return Ok(());
        };

        if let Some(issue) = check_prompt(&prompt) {
            log::warn!("Prompt rejected: {issue:?}");
            ctx.reply(issue).await?;
            return Ok(());
        }

        let status_msg = ctx.reply(format!("Generating {prompt}…")).await?;

        match stablehorde::generate(ctx.http_client.clone(), &prompt).await? {
            Ok(result) => {
                let images = result
                    .images
                    .into_iter()
                    .flat_map(|image| {
                        image::load_from_memory_with_format(&image, ImageFormat::WebP)
                    })
                    .collect::<Vec<_>>();
                let image = image_collage(images, 2, 8);
                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

                ctx.api
                    .make_request(&SendPhoto {
                        chat_id: ctx.message.chat_id(),
                        photo: FileType::Bytes("image.png".to_string(), buffer.into_inner()),
                        caption: Some(format!(
                            "Generated from prompt: *{}* in {}\\.",
                            escape_markdown(prompt),
                            format_duration(result.duration)
                        )),
                        parse_mode: Some(ParseMode::MarkdownV2),
                        reply_to_message_id: Some(ctx.message.message_id),
                        allow_sending_without_reply: Some(true),
                        reply_markup: Some(ReplyMarkup::InlineKeyboardMarkup(
                            InlineKeyboardMarkup {
                                inline_keyboard: vec![vec![InlineKeyboardButton {
                                    text: "Generated thanks to Stable Horde".to_string(),
                                    url: Some("https://stablehorde.net/".to_string()),
                                    ..Default::default()
                                }]],
                            },
                        )),
                    })
                    .await?;
            }
            Err(err) => {
                ctx.reply(err).await?;
            }
        };

        ctx.delete_message(&status_msg).await?;

        Ok(())
    }
}
