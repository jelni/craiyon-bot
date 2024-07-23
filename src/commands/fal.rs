use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileRemote, InputMessagePhoto};

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
    submodel_name: Option<&'static str>,
    num_inference_steps: u8,
}

impl Fal {
    pub const fn realistic_vision() -> Self {
        Self {
            command_names: &["realistic_vision", "rv"],
            description: "generate images using Realistic Vision",
            model_name: "realistic-vision",
            submodel_name: Some("SG161222/Realistic_Vision_V6.0_B1_noVAE"),
            num_inference_steps: 35,
        }
    }

    pub const fn sdxl_lightning() -> Self {
        Self {
            command_names: &["sdxl_lightning", "sdxl"],
            description: "generate images using SDXL Lightning",
            model_name: "fast-lightning-sdxl",
            submodel_name: None,
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
            submodel_name: self.submodel_name,
            prompt,
            negative_prompt: String::new(),
            image_size: ImageSize { height: 1024, width: 1024 },
            num_inference_steps: self.num_inference_steps,
            expand_prompt: false,
            guidance_scale: 5,
            num_images: 1,
            enable_safety_checker: false,
            format: "png",
        };

        let response = generate(&ctx.bot_state.http_client, request).await?;

        let message = ctx
            .reply_custom(
                InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                    photo: InputFile::Remote(InputFileRemote {
                        id: response.images[0].url.clone(),
                    }),
                    thumbnail: None,
                    added_sticker_file_ids: Vec::new(),
                    width: 0,
                    height: 0,
                    caption: Some(formatted_text(vec![
                        "generated ".text(),
                        response.prompt.text(),
                    ])),
                    show_caption_above_media: false,
                    self_destruct_type: None,
                    has_spoiler: false,
                }),
                None,
            )
            .await?;

        ctx.bot_state.message_queue.wait_for_message(message.id).await?;

        Ok(())
    }
}
