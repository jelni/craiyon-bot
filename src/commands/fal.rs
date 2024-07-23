use std::env;
use std::io::BufWriter;

use async_trait::async_trait;
use image::{DynamicImage, ImageFormat};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tdlib::enums::{InlineKeyboardButtonType, InputFile, InputMessageContent, ReplyMarkup};
use tdlib::types::{
    FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, InputFileLocal,
    InputMessagePhoto, ReplyMarkupInlineKeyboard,
};
use tempfile::NamedTempFile;

use crate::commands::{CommandError, CommandResult, CommandTrait};
use crate::utilities::api_utils::DetectServerError;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{formatted_text, ToEntity};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils;

pub struct Fal {
    command_names: &'static [&'static str],
    description: &'static str,
    model_name: &'static str,
    size: (u16, u16),
    num_inference_steps: u8,
}

impl Fal {
    pub const fn realistic_vision() -> Self {
        Self {
            command_names: &["img", "rv", "image"], // this is the main model
            description: "generate images using Realistic Vision",
            model_name: "SG161222/RealVisXL_V4.0",
            size: (1024, 1024),
            num_inference_steps: 6,
        }
    }

    pub const fn sdxl_lightning() -> Self {
        Self {
            command_names: &["sdxl"],
            description: "generate images using SDXL Lightning",
            model_name: "fast-lightning-sdxl",
            size: (1024, 1024),
            num_inference_steps: 4,
        }
    }
}

#[async_trait]
impl CommandTrait for Fal {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            Err(issue)?;
        }

        ctx.send_typing().await?;

        let generation = self.generate(ctx, prompt).await.map_err(|e| {
            log::error!("fal.ai error: {:#?}", e);
            e
        })?;
        let image = process_image(&generation);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Jpeg).unwrap();

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
                        text: String::from("generated by fal.ai"),
                        r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl {
                            url: "https://fal.ai/".into(),
                        }),
                    }]],
                })),
            )
            .await?;

        ctx.bot_state.message_queue.wait_for_message(message.id).await?;
        temp_file.close().unwrap();

        Ok(())
    }
}

impl Fal {
    async fn generate(
        &self,
        ctx: &CommandContext,
        prompt: String,
    ) -> Result<Generation, CommandError> {
        let request = FalRequest {
            model_name: self.model_name,
            prompt,
            negative_prompt: String::new(),
            image_size: ImageSize {
                height: self.size.0,
                width: self.size.1,
            },
            num_inference_steps: self.num_inference_steps,
            guidance_scale: 5,
            loras: vec![],
            embeddings: vec![],
            num_images: 1, // this variable is potentially changable - this is so cheap anyways
            enable_safety_checker: true,
            format: "jpeg",
            safety_checker_version: "v1",
        };

        let response = ctx
            .bot_state
            .http_client
            .post(format!("https://fal.run/fal-ai/{}", self.model_name))
            .header("Authorization", format!("Key {}", env::var("FAL_API_KEY").unwrap()))
            .json(&request)
            .send()
            .await?
            .server_error()?;

        if response.status() == StatusCode::OK {
            let response = response.json::<FalResponse>().await?;
            Ok(Generation {
                image: response.images[0].clone(),
            })
        } else {
            let response = response.json::<ErrorResponse>().await?;
            Err(CommandError::Custom(response.error.message))
        }
    }
}

struct Generation {
    image: String,
}

fn process_image(generation: &Generation) -> DynamicImage {
    let image = image::load_from_memory_with_format(generation.image.as_bytes(), ImageFormat::Jpeg)
        .unwrap();
    image
}

fn format_result_text(generation: Generation) -> FormattedText {
    let entities = vec!["generated ".text(), generation.image.text()];
    formatted_text(entities)
}

#[derive(Serialize)]
pub struct FalRequest {
    pub model_name: &'static str,
    pub prompt: String,
    pub negative_prompt: String,
    pub image_size: ImageSize,
    pub num_inference_steps: u8,
    pub guidance_scale: u8,
    pub loras: Vec<String>,
    pub embeddings: Vec<String>,
    pub num_images: u8,
    pub enable_safety_checker: bool,
    pub format: &'static str,
    pub safety_checker_version: &'static str,
}

#[derive(Serialize)]
pub struct ImageSize {
    pub height: u16,
    pub width: u16,
}

#[derive(Deserialize)]
pub struct FalResponse {
    pub images: Vec<String>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}

#[derive(Deserialize)]
pub struct Error {
    // pub code: String,
    pub message: String,
}