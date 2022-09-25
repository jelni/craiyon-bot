use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use image::{ImageFormat, ImageOutputFormat};
use reqwest::StatusCode;
use tgbotapi::requests::ParseMode;
use tgbotapi::FileType;

use super::CommandTrait;
use crate::api_methods::SendPhoto;
use crate::apis::craiyon;
use crate::ratelimit::RateLimiter;
use crate::utils::{
    check_prompt, donate_markup, escape_markdown, format_duration, image_collage, Context,
};

#[derive(Default)]
pub struct Generate;

#[async_trait]
impl CommandTrait for Generate {
    fn name(&self) -> &str {
        "generate"
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60)
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

        match craiyon::generate(ctx.http_client.clone(), &prompt).await {
            Ok(result) => {
                let images = result
                    .images
                    .into_iter()
                    .flat_map(|image| {
                        image::load_from_memory_with_format(&image, ImageFormat::Jpeg)
                    })
                    .collect::<Vec<_>>();
                let image = image_collage(images, 3, 8);
                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

                ctx.api
                    .make_request(&SendPhoto {
                        chat_id: ctx.message.chat_id(),
                        photo: FileType::Bytes("image.png".to_string(), buffer.into_inner()),
                        caption: Some(format!(
                            "Generated *{}* in {}\\.",
                            escape_markdown(prompt),
                            format_duration(result.duration.as_secs())
                        )),
                        parse_mode: Some(ParseMode::MarkdownV2),
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

        ctx.delete_message(&status_msg).await?;

        Ok(())
    }
}
