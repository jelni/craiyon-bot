use std::error::Error;
use std::io::Cursor;

use async_trait::async_trait;
use image::{ImageFormat, ImageOutputFormat};
use reqwest::StatusCode;
use tgbotapi::requests::{DeleteMessage, SendMessage, SendPhoto};
use tgbotapi::FileType;

use super::Command;
use crate::utils::{donate_markup, CollageOptions, Context};
use crate::{craiyon, utils};

const DISALLOWED_WORDS: [&str; 18] = [
    "18+", "abuse", "anus", "ass", "boob", "boobs", "breast", "breasts", "butt", "butts", "erotic",
    "loli", "lolicon", "naked", "nude", "penis", "sex", "sexy",
];

pub struct Generate;

#[async_trait]
impl Command for Generate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let prompt = match ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("prompt to generate").await;
                return Ok(());
            }
        };

        if prompt.chars().count() > 1024 {
            ctx.api
                .make_request(&SendMessage {
                    chat_id: ctx.message.chat_id(),
                    text: "This prompt is too long.".to_string(),
                    reply_to_message_id: Some(ctx.message.message_id),
                    ..Default::default()
                })
                .await?;

            return Ok(());
        };

        if is_prompt_suspicious(&prompt) {
            log::warn!("Suspicious prompt rejected");
            ctx.api
                .make_request(&SendMessage {
                    chat_id: ctx.message.chat_id(),
                    text: "This prompt is sus.".to_string(),
                    reply_to_message_id: Some(ctx.message.message_id),
                    ..Default::default()
                })
                .await?;

            return Ok(());
        }

        let status_msg = ctx
            .api
            .make_request(&SendMessage {
                chat_id: ctx.message.chat_id(),
                text: format!("Generating {prompt}…"),
                reply_to_message_id: Some(ctx.message.message_id),
                ..Default::default()
            })
            .await?
            .message_id;

        match craiyon::generate(ctx.http_client, prompt.clone()).await {
            Ok(result) => {
                let images = result
                    .images
                    .iter()
                    .flat_map(|image| image::load_from_memory_with_format(image, ImageFormat::Jpeg))
                    .collect::<Vec<_>>();
                let image_size = {
                    let image = images.first().unwrap();
                    (image.width(), image.height())
                };
                let image = utils::image_collage(
                    images,
                    CollageOptions { image_count: (3, 3), image_size, gap: 8 },
                );

                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

                ctx.api
                    .make_request(&SendPhoto {
                        chat_id: ctx.message.chat_id(),
                        photo: FileType::Bytes("image.png".to_string(), buffer.into_inner()),
                        // missing `parse_mode`!
                        // caption: Some(format!(
                        //     "Generated from prompt: *{}* in {}\\.",
                        //     escape_markdown(prompt),
                        //     utils::format_duration(result.duration)
                        // )),
                        caption: Some(format!(
                            "Generated from prompt: {prompt} in {}.",
                            utils::format_duration(result.duration)
                        )),
                        reply_to_message_id: Some(ctx.message.message_id),
                        allow_sending_without_reply: Some(true),
                        reply_markup: Some(donate_markup(
                            "🖍️ Craiyon",
                            "https://www.craiyon.com/donate",
                        )),
                    })
                    .await?;
            }
            Err(err) => {
                ctx.api
                    .make_request(&SendMessage {
                        chat_id: ctx.message.chat_id(),
                        text: format!(
                            "zjebalo sie: {}",
                            err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                        ),
                        reply_to_message_id: Some(ctx.message.message_id),
                        allow_sending_without_reply: Some(true),
                        ..Default::default()
                    })
                    .await?;
            }
        };

        ctx.api
            .make_request(&DeleteMessage { chat_id: ctx.message.chat_id(), message_id: status_msg })
            .await
            .ok();

        Ok(())
    }
}

fn is_prompt_suspicious<S: AsRef<str>>(text: S) -> bool {
    text.as_ref()
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .any(|w| DISALLOWED_WORDS.contains(&w))
}
