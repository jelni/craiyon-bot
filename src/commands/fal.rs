use std::io::Write;

use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;

use crate::apis::fal::{generate, FalRequest, ImageSize};
use crate::commands::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{formatted_text, ToEntity};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils;

pub struct Fal {
    command_names: &'static [&'static str],
    description: &'static str,
    model_name: &'static str,
    num_inference_steps: u8,
}

impl Fal {
    pub const fn realistic_vision() -> Self {
        Self {
            command_names: &["realistic_vision", "rv"],
            description: "generate images using Realistic Vision",
            model_name: "SG161222/RealVisXL_V4.0",
            num_inference_steps: 6,
        }
    }

    pub const fn sdxl_lightning() -> Self {
        Self {
            command_names: &["sdxl_lightning", "sdxl"],
            description: "generate images using SDXL Lightning",
            model_name: "fast-lightning-sdxl",
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

        let request = FalRequest {
            model_name: self.model_name,
            prompt,
            negative_prompt: String::new(),
            image_size: ImageSize { height: 1024, width: 1024 },
            num_inference_steps: self.num_inference_steps,
            guidance_scale: 5,
            num_images: 1,
            enable_safety_checker: true,
            format: "png",
        };

        let response = generate(&ctx.bot_state.http_client, request).await?;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(response.images[0].as_bytes()).unwrap();

        let message = ctx
            .reply_custom(
                InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                    photo: InputFile::Local(InputFileLocal {
                        path: temp_file.path().to_str().unwrap().into(),
                    }),
                    thumbnail: None,
                    added_sticker_file_ids: Vec::new(),
                    width: 0,
                    height: 0,
                    caption: Some(formatted_text(vec!["generated ".text(), response.images[0].text()])),
                    show_caption_above_media: false,
                    self_destruct_type: None,
                    has_spoiler: false,
                }),
                None,
            )
            .await?;

        ctx.bot_state.message_queue.wait_for_message(message.id).await?;
        temp_file.close().unwrap();

        Ok(())
    }
}
