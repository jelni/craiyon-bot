use std::error::Error;
use std::io::Cursor;

use async_trait::async_trait;
use image::{ImageFormat, ImageOutputFormat};
use reqwest::StatusCode;
use tgbotapi::requests::{DeleteMessage, SendPhoto};
use tgbotapi::FileType;

use super::Command;
use crate::utils::{donate_markup, CollageOptions, Context};
use crate::{craiyon, utils};

// yes, people generated all of these
#[rustfmt::skip]
const DISALLOWED_WORDS: [&str; 27] = [
    "18+", "abuse", "anus", "ass", "boob", "boobs", "breast", "breasts", "butt", "butts", "erotic",
    "hentai", "incest", "loli", "lolicon", "lolis", "naked", "nude", "penis", "porn", "porno",
    "rape", "sex", "sexy", "shota", "shotacon", "suggestive",
];

pub struct Generate;

#[async_trait]
impl Command for Generate {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prompt = match &ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("prompt to generate").await;
                return Ok(());
            }
        };

        if prompt.chars().count() > 1024 {
            ctx.reply("This prompt is too long.").await?;

            return Ok(());
        };

        if is_prompt_suspicious(prompt) {
            log::warn!("Suspicious prompt rejected");
            ctx.reply("This prompt is sus.").await?;

            return Ok(());
        }

        let status_msg = ctx.reply(format!("Generating {prompt}…")).await?.message_id;

        match craiyon::generate(ctx.http_client.clone(), prompt.clone()).await {
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
                ctx.reply(format!(
                    "zjebalo sie: {}",
                    err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                ))
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
