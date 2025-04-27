use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use log::error;
use std::io::Write;
use tempfile::NamedTempFile;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{FormattedText, InputFileLocal, InputMessagePhoto};

use crate::commands::{CommandError, CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::text_utils;
use crate::utilities::together::{TogetherClient, TogetherImageRequest};

pub struct Flux;

#[async_trait]
impl CommandTrait for Flux {
    fn command_names(&self) -> &[&str] {
        &["flux"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("generate an image using FLUX[schnell]")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        if let Some(issue) = text_utils::check_prompt(&prompt) {
            log::info!("prompt rejected: {issue:?}");
            return Err(CommandError::Custom(issue.into()));
        }

        ctx.send_typing().await?;

        let client = TogetherClient::new();
        let request = TogetherImageRequest {
            model: "black-forest-labs/FLUX.1-schnell-Free".to_string(),
            prompt: prompt.clone(),
            width: 1024,
            height: 768,
            steps: 4,
            n: 1,
            response_format: "b64_json".to_string(),
        };

        let start = std::time::Instant::now();
        let response = client.generate_image(request).await;
        let elapsed = start.elapsed();
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                error!("Together API error: {e}");
                ctx.reply(format!("Together API error: {e}")).await?;
                return Ok(());
            }
        };

        let base64 = &response.data[0].b64_json;
        let image_bytes = BASE64.decode(base64).map_err(|e| CommandError::Custom(e.to_string()))?;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&image_bytes).unwrap();

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
                    caption: Some(FormattedText {
                        text: format!("generated with FLUX: {}\n({:.2}s)", prompt, elapsed.as_secs_f32()),
                        ..Default::default()
                    }),
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
