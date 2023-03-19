use std::fmt::Write;
use std::io::BufWriter;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use counter::Counter;
use image::{DynamicImage, ImageFormat};
use reqwest::Url;
use tdlib::enums::{
    FormattedText, InlineKeyboardButtonType, InputFile, InputMessageContent, ReplyMarkup,
    TextParseMode,
};
use tdlib::functions;
use tdlib::types::{
    InlineKeyboardButton, InlineKeyboardButtonTypeUrl, InputFileLocal, InputMessagePhoto, Message,
    ReplyMarkupInlineKeyboard, TextParseModeMarkdown,
};
use tempfile::NamedTempFile;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::stablehorde::{self, Generation, Status};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedyOrReply;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils::{EscapeMarkdown, TruncateWithEllipsis};
use crate::utilities::{api_utils, image_utils, text_utils};

pub struct StableHorde {
    command_names: &'static [&'static str],
    command_description: &'static str,
    model: &'static str,
    size: (u32, u32),
}

impl StableHorde {
    pub fn stable_diffusion_2() -> Self {
        Self {
            command_names: &["stable_diffusion_2", "sd2"],
            command_description: "generate images using Stable Diffusion v2.1",
            model: "stable_diffusion_2.1",
            size: (768, 768),
        }
    }

    pub fn stable_diffusion() -> Self {
        Self {
            command_names: &["stable_diffusion", "sd"],
            command_description: "generate images using Stable Diffusion v1.5",
            model: "stable_diffusion",
            size: (512, 512),
        }
    }

    pub fn waifu_diffusion() -> Self {
        Self {
            command_names: &["waifu_diffusion", "wd"],
            command_description: "generate images using Waifu Diffusion",
            model: "waifu_diffusion",
            size: (512, 512),
        }
    }

    pub fn furry_diffusion() -> Self {
        Self {
            command_names: &["furry_diffusion", "fd"],
            command_description: "generate images using Furry Epoch",
            model: "Furry Epoch",
            size: (512, 512),
        }
    }
}

#[async_trait]
impl CommandTrait for StableHorde {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.command_description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 300)
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            Err(issue)?;
        }

        ctx.send_typing().await?;

        let generation = self.generate(ctx.clone(), prompt).await?;
        let mut temp_file = NamedTempFile::new().unwrap();
        generation.image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Png).unwrap();

        let FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            format_result_text(
                generation.time_taken,
                &generation.workers,
                &generation.escaped_prompt,
            ),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            ctx.client_id,
        )
        .await?;

        let message = ctx
            .reply_custom(
                InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                    photo: InputFile::Local(InputFileLocal {
                        path: temp_file.path().to_str().unwrap().into(),
                    }),
                    thumbnail: None,
                    added_sticker_file_ids: Vec::new(),
                    width: generation.image.width().try_into().unwrap(),
                    height: generation.image.height().try_into().unwrap(),
                    caption: Some(formatted_text),
                    self_destruct_time: 0,
                    has_spoiler: false,
                }),
                Some(ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
                    rows: vec![vec![InlineKeyboardButton {
                        text: "generated thanks to Stable Horde".into(),
                        r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl {
                            url: "https://stablehorde.net/".into(),
                        }),
                    }]],
                })),
            )
            .await?;

        ctx.message_queue.wait_for_message(message.id).await?;
        if let Some(status_msg) = generation.status_msg {
            ctx.delete_message(status_msg.id).await.ok();
        }
        temp_file.close().unwrap();

        Ok(())
    }
}

struct GenerationResult {
    image: DynamicImage,
    time_taken: Duration,
    escaped_prompt: String,
    workers: Counter<String>,
    status_msg: Option<Message>,
}

impl StableHorde {
    async fn generate(
        &self,
        ctx: Arc<CommandContext>,
        prompt: String,
    ) -> Result<GenerationResult, CommandError> {
        let request_id =
            stablehorde::generate(ctx.http_client.clone(), &prompt, self.model, self.size)
                .await??;
        let escaped_prompt = EscapeMarkdown(&prompt).to_string();
        let (results, status_msg, time_taken) =
            wait_for_generation(ctx.clone(), &request_id, &escaped_prompt).await?;
        let workers =
            results.iter().map(|generation| generation.worker_name.clone()).collect::<Counter<_>>();
        let urls = results
            .into_iter()
            .filter_map(|generation| {
                if let Ok(url) = api_utils::cloudflare_storage_url(&generation.img) {
                    Some(url)
                } else {
                    log::error!(
                        "worker {} {:?} returned invalid image data: {}",
                        generation.worker_id,
                        generation.worker_name,
                        generation.img.clone().truncate_with_ellipsis(256)
                    );
                    None
                }
            })
            .collect::<Vec<_>>();
        if urls.is_empty() {
            Err("no images were successfully generated.")?;
        }
        let images = download_images(ctx.http_client.clone(), urls).await?;
        let image = process_images(images, self.size);

        Ok(GenerationResult { image, time_taken, escaped_prompt, workers, status_msg })
    }
}

async fn wait_for_generation(
    ctx: Arc<CommandContext>,
    request_id: &str,
    escaped_prompt: &str,
) -> Result<(Vec<Generation>, Option<Message>, Duration), CommandError> {
    let start_time = Instant::now();
    let mut status_msg: Option<Message> = None;
    let mut last_edit: Option<Instant> = None;
    let mut last_status = None;
    let mut show_volunteer_notice = false;
    let time_taken = loop {
        let status = stablehorde::check(ctx.http_client.clone(), request_id).await??;

        if status.done {
            break start_time.elapsed();
        };

        if status.faulted {
            Err("the generation timed out.")?;
        }

        if !status.is_possible {
            stablehorde::cancel_generation(ctx.http_client.clone(), request_id).await?;
            Err("there are no online workers for the requested model.")?;
        }

        if status.wait_time >= 60 {
            show_volunteer_notice = true;
        }

        if last_status.as_ref() != Some(&status) {
            // the message doesn't exist yet or was edited more than 12 seconds ago
            if last_edit.map_or(true, |last_edit| last_edit.elapsed() >= Duration::from_secs(12)) {
                let text = format_status_text(&status, escaped_prompt, show_volunteer_notice);
                status_msg = Some(match status_msg {
                    None => {
                        ctx.message_queue
                            .wait_for_message(ctx.reply_markdown(text).await?.id)
                            .await?
                    }
                    Some(status_msg) => ctx.edit_message_markdown(status_msg.id, text).await?,
                });

                last_edit = Some(Instant::now());
                last_status = Some(status);
            }
        };

        tokio::time::sleep(Duration::from_secs(2)).await;
    };

    let results = stablehorde::results(ctx.http_client.clone(), request_id).await??;
    Ok((results, status_msg, time_taken))
}

async fn download_images(
    http_client: reqwest::Client,
    urls: Vec<Url>,
) -> Result<Vec<Vec<u8>>, CommandError> {
    let mut images = Vec::with_capacity(urls.len());
    let tasks = urls.into_iter().map(|url| tokio::spawn(http_client.get(url).send()));
    for task in tasks {
        images.push(task.await.unwrap()?.bytes().await.unwrap().to_vec());
    }

    Ok(images)
}

fn process_images(images: Vec<Vec<u8>>, size: (u32, u32)) -> DynamicImage {
    let images = images
        .into_iter()
        .flat_map(|image| image::load_from_memory_with_format(&image, ImageFormat::WebP))
        .collect();

    image_utils::collage(images, size, 8)
}

fn format_status_text(status: &Status, escaped_prompt: &str, volunteer_notice: bool) -> String {
    let queue_info = if status.queue_position > 0 {
        format!("queue position: {}\n", status.queue_position)
    } else {
        String::new()
    };

    let mut text = format!(
        "generating {escaped_prompt}…\n{queue_info}`{}` ETA: {}",
        progress_bar(
            status.waiting.unsigned_abs() as usize,
            status.processing.unsigned_abs() as usize,
            status.finished.unsigned_abs() as usize
        ),
        text_utils::format_duration(status.wait_time.try_into().unwrap())
    );

    if volunteer_notice {
        text.push_str(concat!(
            "\n\nStable Horde is run by volunteers\\. ",
            "to make waiting times shorter, ",
            "[consider joining yourself](https://stablehorde.net/)\\!"
        ));
    };

    text
}

fn format_result_text(
    time_taken: Duration,
    workers: &Counter<String>,
    escaped_prompt: &str,
) -> String {
    format!(
        "generated *{}* in {} by {}\\.",
        escaped_prompt,
        text_utils::format_duration(time_taken.as_secs()),
        workers
            .most_common()
            .into_iter()
            .map(|(worker_name, generated_images)| {
                let mut worker_name =
                    EscapeMarkdown(&worker_name.truncate_with_ellipsis(64)).to_string();
                if generated_images > 1 {
                    write!(worker_name, " \\({generated_images}\\)").unwrap();
                }
                worker_name
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
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
