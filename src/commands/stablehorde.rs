use std::io::BufWriter;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use counter::Counter;
use image::{DynamicImage, ImageFormat};
use reqwest::Url;
use tdlib::enums::{InlineKeyboardButtonType, InputFile, InputMessageContent, ReplyMarkup};
use tdlib::types::{
    FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, InputFileLocal,
    InputMessagePhoto, ReplyMarkupInlineKeyboard,
};
use tempfile::NamedTempFile;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::stablehorde::{self, GeneratedImage, Status};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{self, formatted_text, ToEntity, ToEntityOwned};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils::TruncateWithEllipsis;
use crate::utilities::{api_utils, image_utils, text_utils};

pub struct StableHorde {
    command_names: &'static [&'static str],
    description: &'static str,
    model: &'static str,
    size: (u32, u32),
}

impl StableHorde {
    pub const fn stable_diffusion_2() -> Self {
        Self {
            command_names: &["stable_diffusion_2", "sd2"],
            description: "generate images using Stable Diffusion v2.1",
            model: "stable_diffusion_2.1",
            size: (512, 512),
        }
    }

    pub const fn stable_diffusion() -> Self {
        Self {
            command_names: &["stable_diffusion", "sd"],
            description: "generate images using Stable Diffusion v1.5",
            model: "stable_diffusion",
            size: (512, 512),
        }
    }

    pub const fn waifu_diffusion() -> Self {
        Self {
            command_names: &["waifu_diffusion", "wd"],
            description: "generate images using Waifu Diffusion",
            model: "waifu_diffusion",
            size: (512, 512),
        }
    }

    pub const fn furry_diffusion() -> Self {
        Self {
            command_names: &["furry_diffusion", "fd"],
            description: "generate images using Furry Epoch",
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
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 300)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            Err(issue)?;
        }

        ctx.send_typing().await?;

        let generation = Box::pin(self.generate(ctx, prompt)).await?;
        let images = download_images(ctx.bot_state.http_client.clone(), &generation.urls).await?;
        let image = process_images(images, self.size);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Png).unwrap();

        let status_msg_id = generation.status_msg_id;

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
                    caption: Some(format_result_text(generation)),
                    show_caption_above_media: false,
                    self_destruct_type: None,
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

        ctx.bot_state.message_queue.wait_for_message(message.id).await?;
        if let Some(status_msg_id) = status_msg_id {
            ctx.delete_message(status_msg_id).await.ok();
        }
        temp_file.close().unwrap();

        Ok(())
    }
}

struct Generation {
    urls: Vec<Url>,
    time_taken: Duration,
    escaped_prompt: String,
    workers: Counter<String>,
    status_msg_id: Option<i64>,
}

impl StableHorde {
    async fn generate(
        &self,
        ctx: &CommandContext,
        prompt: String,
    ) -> Result<Generation, CommandError> {
        let request_id = stablehorde::generate(
            ctx.bot_state.http_client.clone(),
            &prompt,
            self.model,
            self.size,
        )
        .await??;
        let escaped_prompt = prompt.truncate_with_ellipsis(256);
        let (results, status_msg_id, time_taken) =
            Box::pin(wait_for_generation(ctx, &request_id, &escaped_prompt)).await?;
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

        Ok(Generation { urls, time_taken, escaped_prompt, workers, status_msg_id })
    }
}


async fn wait_for_generation(
    ctx: &CommandContext,
    request_id: &str,
    escaped_prompt: &str,
) -> Result<(Vec<GeneratedImage>, Option<i64>, Duration), CommandError> {
    let start_time = Instant::now();
    let mut status_msg_id: Option<i64> = None;
    let mut last_edit: Option<Instant> = None;
    let mut last_status = None;
    let mut show_volunteer_notice = false;

    let time_taken = loop {
        let status = stablehorde::check(ctx.bot_state.http_client.clone(), request_id).await??;

        if status.done {
            break start_time.elapsed();
        };

        if status.faulted {
            Err("the generation timed out.")?;
        }

        if !status.is_possible {
            stablehorde::cancel_generation(ctx.bot_state.http_client.clone(), request_id).await?;
            Err("there are no online workers for the requested model.")?;
        }

        if status.wait_time >= 60 {
            show_volunteer_notice = true;
        }

        if last_status.as_ref() != Some(&status) {
            // the message doesn't exist yet or was edited more than 12 seconds ago
            if last_edit.map_or(true, |last_edit| last_edit.elapsed() >= Duration::from_secs(12)) {
                let formatted_text =
                    format_status_text(&status, escaped_prompt, show_volunteer_notice);
                status_msg_id = Some(match status_msg_id {
                    None => {
                        ctx.bot_state
                            .message_queue
                            .wait_for_message(ctx.reply_formatted_text(formatted_text).await?.id)
                            .await?
                            .id
                    }
                    Some(status_msg) => {
                        ctx.edit_message_formatted_text(status_msg, formatted_text).await?.id
                    }
                });

                last_edit = Some(Instant::now());
                last_status = Some(status);
            }
        };

        tokio::time::sleep(Duration::from_secs(2)).await;
    };

    let results = stablehorde::results(ctx.bot_state.http_client.clone(), request_id).await??;
    Ok((results, status_msg_id, time_taken))
}

async fn download_images(
    http_client: reqwest::Client,
    urls: &[Url],
) -> Result<Vec<Vec<u8>>, CommandError> {
    let tasks = urls.iter().map(|url| tokio::spawn(http_client.get(url.clone()).send()));

    let mut images = Vec::with_capacity(tasks.len());
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

fn format_status_text(
    status: &Status,
    escaped_prompt: &str,
    volunteer_notice: bool,
) -> FormattedText {
    let queue_info = if status.queue_position > 0 {
        format!("queue position: {}\n", status.queue_position)
    } else {
        String::new()
    };

    let mut entities = vec![
        "generating ".text(),
        escaped_prompt.text(),
        "…\n".text(),
        queue_info.text(),
        progress_bar(
            status.waiting.unsigned_abs() as usize,
            status.processing.unsigned_abs() as usize,
            status.finished.unsigned_abs() as usize,
        )
        .code_owned(),
        " ETA: ".text(),
        text_utils::format_duration(status.wait_time.into()).text_owned(),
    ];

    if volunteer_notice {
        entities.extend([
            "\n\nStable Horde is run by volunteers. to make wait times shorter, ".text(),
            "consider joining yourself".text_url("https://stablehorde.net/"),
            "!".text(),
        ]);
    };

    message_entities::formatted_text(entities)
}

fn format_result_text(generation: Generation) -> FormattedText {
    let workers = generation
        .workers
        .most_common()
        .into_iter()
        .flat_map(|(worker_name, generated_images)| {
            let mut entities =
                vec![", ".text(), worker_name.truncate_with_ellipsis(64).text_owned()];

            if generated_images > 1 {
                entities.extend([
                    " (".text(),
                    generated_images.to_string().text_owned(),
                    ")".text(),
                ]);
            }

            entities
        })
        .skip(1)
        .collect::<Vec<_>>();

    let download_urls = generation
        .urls
        .into_iter()
        .enumerate()
        .flat_map(|(i, url)| [" ".text(), (i + 1).to_string().text_url_owned(url.to_string())])
        .skip(1)
        .collect::<Vec<_>>();

    let mut entities = vec![
        "generated ".text(),
        generation.escaped_prompt.bold(),
        " in ".text(),
        text_utils::format_duration(generation.time_taken.as_secs()).text_owned(),
        " by ".text(),
    ];

    entities.extend(workers);
    entities.push(".\ndownload: ".text());
    entities.extend(download_urls);

    formatted_text(entities)
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
