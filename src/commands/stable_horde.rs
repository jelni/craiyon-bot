use std::error::Error;
use std::fmt::Write;
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use counter::Counter;
use image::{ImageFormat, ImageOutputFormat};
use tgbotapi::requests::{ParseMode, ReplyMarkup};
use tgbotapi::{FileType, InlineKeyboardButton, InlineKeyboardMarkup};

use super::CommandTrait;
use crate::api_methods::SendPhoto;
use crate::apis::stablehorde::{self, Status};
use crate::ratelimit::RateLimiter;
use crate::utils::{
    check_prompt, escape_markdown, format_duration, image_collage, Context, TruncateWithEllipsis,
};

pub struct StableHorde {
    command_name: &'static str,
    command_aliases: &'static [&'static str],
    models: &'static [&'static str],
}

impl StableHorde {
    pub fn stable_diffusion() -> Self {
        Self {
            command_name: "stable_diffusion",
            command_aliases: &["sd"],
            models: &["stable_diffusion"],
        }
    }

    pub fn waifu_diffusion() -> Self {
        Self {
            command_name: "waifu_diffusion",
            command_aliases: &["wd"],
            models: &["waifu_diffusion"],
        }
    }

    pub fn furry_diffusion() -> Self {
        Self {
            command_name: "furry_diffusion",
            command_aliases: &["fd", "furry_epoch", "fe"],
            models: &["Furry Epoch"],
        }
    }
}

#[async_trait]
impl CommandTrait for StableHorde {
    fn name(&self) -> &'static str {
        self.command_name
    }

    fn aliases(&self) -> &[&str] {
        self.command_aliases
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 300)
    }

    #[allow(clippy::too_many_lines)]
    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(prompt) = arguments else {
            ctx.missing_argument("prompt to generate").await;
            return Ok(());
        };

        if let Some(issue) = check_prompt(&prompt) {
            log::info!("Prompt rejected: {issue:?}");
            ctx.reply(issue).await?;
            return Ok(());
        }

        let request_id =
            match stablehorde::generate(ctx.http_client.clone(), self.models, &prompt).await? {
                Ok(request_id) => request_id,
                Err(err) => {
                    ctx.reply(err).await?;
                    return Ok(());
                }
            };

        let mut status_msg = None;
        let escaped_prompt = escape_markdown(prompt);
        let start = Instant::now();
        let mut first_wait_time = 0;
        let mut last_edit: Option<Instant> = None;
        let mut last_status = None;
        loop {
            let status = match stablehorde::check(ctx.http_client.clone(), &request_id).await? {
                Ok(status) => status,
                Err(err) => {
                    ctx.reply(err).await?;
                    return Ok(());
                }
            };

            if status.done {
                break;
            };

            if first_wait_time == 0 {
                first_wait_time = status.wait_time;
            }

            if last_status.as_ref() != Some(&status) {
                // the message doesn't exist yet or was edited more than 12 seconds ago
                if last_edit
                    .map_or(true, |last_edit| last_edit.elapsed() >= Duration::from_secs(12))
                {
                    let text = format_status(&status, &escaped_prompt, first_wait_time);
                    match &status_msg {
                        None => {
                            status_msg = Some(ctx.reply_markdown(text).await?);
                        }
                        Some(status_msg) => {
                            ctx.edit_message_markdown(status_msg, text).await?;
                        }
                    };

                    last_edit = Some(Instant::now());
                    last_status = Some(status);
                }
            };

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        let duration = start.elapsed();

        let results = match stablehorde::results(ctx.http_client.clone(), &request_id).await? {
            Ok(results) => results,
            Err(err) => {
                ctx.reply(err).await?;
                return Ok(());
            }
        };
        let mut workers = Counter::<String>::new();
        let images = results
            .into_iter()
            .flat_map(|generation| {
                workers[&generation.worker_name] += 1;
                base64::decode(generation.img)
            })
            .flat_map(|image| image::load_from_memory_with_format(&image, ImageFormat::WebP))
            .collect::<Vec<_>>();

        let image = image_collage(images, (512, 512), 2, 8);
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

        ctx.api
            .make_request(&SendPhoto {
                chat_id: ctx.message.chat_id(),
                photo: FileType::Bytes("image.png".to_string(), buffer.into_inner()),
                caption: Some(format!(
                    "Generated *{}* in {} by {}\\.",
                    escaped_prompt,
                    format_duration(duration.as_secs()),
                    workers
                        .most_common()
                        .into_iter()
                        .map(|(mut k, v)| {
                            k.truncate_with_ellipsis(64);
                            if v > 1 {
                                write!(k, " ({v})").unwrap();
                            }
                            escape_markdown(k)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                parse_mode: Some(ParseMode::MarkdownV2),
                reply_to_message_id: Some(ctx.message.message_id),
                allow_sending_without_reply: Some(true),
                reply_markup: Some(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                    inline_keyboard: vec![vec![InlineKeyboardButton {
                        text: "Generated thanks to Stable Horde".to_string(),
                        url: Some("https://stablehorde.net/".to_string()),
                        ..Default::default()
                    }]],
                })),
            })
            .await?;

        if let Some(status_msg) = status_msg {
            ctx.delete_message(&status_msg).await.ok();
        }

        Ok(())
    }
}

fn format_status(status: &Status, escaped_prompt: &str, first_wait_time: u32) -> String {
    let queue_info = if status.queue_position > 0 {
        format!("Queue position: {}\n", status.queue_position)
    } else {
        String::new()
    };

    let mut text = format!(
        "Generating {escaped_prompt}…\n{queue_info}`{}` ETA: {}",
        progress_bar(status.waiting as usize, status.processing as usize, status.finished as usize),
        format_duration(status.wait_time.try_into().unwrap())
    );

    if first_wait_time >= 30 {
        text.push_str(concat!(
            "\n\nStable Horde is run by volunteers\\. ",
            "To make waiting times shorter, ",
            "[consider joining the horde yourself](https://stablehorde.net/)\\!"
        ));
    };

    text
}

fn progress_bar(waiting: usize, processing: usize, finished: usize) -> String {
    let mut bar = String::with_capacity(4 * (waiting + processing + finished) + 2);
    bar.push('[');
    bar.push_str(&"=".repeat(5 * finished));
    bar.push_str(&"-".repeat(5 * processing));
    bar.push_str(&" ".repeat(5 * waiting));
    bar.push(']');
    bar
}
