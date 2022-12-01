use std::fs::{self};
use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::{File, InputFile, InputMessageContent};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;

use super::CommandError::CustomError;
use super::{CommandResult, CommandTrait};
use crate::apis::different_dimension_me;
use crate::command_context::CommandContext;
use crate::utils;

const MEBIBYTE: i64 = 1024 * 1024;

#[derive(Default)]
pub struct DifferentDimensionMe;

#[async_trait]
impl CommandTrait for DifferentDimensionMe {
    fn command_names(&self) -> &[&str] {
        &["different_dimension_me", "ai2d"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("convert a real life photo to Anime style")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, _: Option<String>) -> CommandResult {
        let mut file = utils::get_message_or_reply_image(&ctx.message, ctx.client_id)
            .await
            .ok_or(CustomError("send or reply to an image.".into()))?;

        if !file.local.is_downloading_completed {
            if file.expected_size > 4 * MEBIBYTE {
                Err("the image cannot be larger than 4 MiB.")?;
            }
            File::File(file) =
                functions::download_file(file.id, 1, 0, 0, true, ctx.client_id).await?;
        }

        ctx.send_typing().await?;

        let result = different_dimension_me::process(
            ctx.http_client.clone(),
            fs::read(file.local.path).unwrap(),
        )
        .await?;

        let media = result.map_err(|err| CustomError(err.to_string()))?;
        let response = ctx
            .http_client
            .get(
                media
                    .img_urls
                    .into_iter()
                    .nth(2)
                    .ok_or(CustomError("the generation failed.".into()))?,
            )
            .send()
            .await?;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&response.bytes().await?).unwrap();

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
                    caption: None,
                    ttl: 0,
                }),
                None,
            )
            .await?;

        ctx.message_queue.wait_for_message(message.id).await?;
        temp_file.close().unwrap();

        Ok(())
    }
}
