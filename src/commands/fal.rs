use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileRemote, InputMessagePhoto};

use crate::apis::fal;
use crate::commands::{CommandError, CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{ToEntity, ToEntityOwned, formatted_text};
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils;

pub struct Fal {
    command_names: &'static [&'static str],
    description: &'static str,
    model_name: &'static str,
}

impl Fal {
    pub const fn sana() -> Self {
        Self {
            command_names: &["sana"],
            description: "generate an image using NVIDIA's ️Sana",
            model_name: "fal-ai/sana",
        }
    }

    pub const fn sdxl() -> Self {
        Self {
            command_names: &["sdxl"],
            description: "generate an image using Stable Diffusion XL",
            model_name: "fal-ai/fast-sdxl",
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
        RateLimiter::new(1, 120)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            return Err(CommandError::Custom(issue.into()));
        }

        ctx.send_typing().await?;
        let response =
            fal::generate(ctx.bot_state.http_client.clone(), self.model_name, &prompt).await?;
        let image = response.images.into_iter().next().unwrap();

        ctx.reply_custom(
            InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                photo: InputFile::Remote(InputFileRemote { id: image.url.clone() }),
                thumbnail: None,
                added_sticker_file_ids: Vec::new(),
                width: image.width.try_into().unwrap(),
                height: image.height.try_into().unwrap(),
                caption: Some(formatted_text(vec![
                    "generated ".text(),
                    response.prompt.bold(),
                    format!(" in {:.2}s. ", response.timings.inference).text_owned(),
                    "download".text_url(image.url),
                ])),
                show_caption_above_media: false,
                self_destruct_type: None,
                has_spoiler: false,
            }),
            None,
        )
        .await?;

        Ok(())
    }
}
