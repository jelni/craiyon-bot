use std::fs::{self};
use std::io::BufWriter;

use async_trait::async_trait;
use image::{DynamicImage, ImageFormat};
use tdlib::enums::{File, InputFile, InputMessageContent};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;

use super::{CommandResult, CommandTrait};
use crate::apis::different_dimension_me;
use crate::utilities::command_context::CommandContext;
use crate::utilities::file_download::MEBIBYTE;
use crate::utilities::message_entities::ToEntity;
use crate::utilities::{message_entities, telegram_utils};

pub struct DifferentDimensionMe;

#[async_trait]
impl CommandTrait for DifferentDimensionMe {
    fn command_names(&self) -> &[&str] {
        &["different_dimension_me", "ai2d", "2d"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        let message_image =
            telegram_utils::get_message_or_reply_attachment(&ctx.message, false, ctx.client_id)
                .await?
                .ok_or("send or reply to an image.")?;

        let file = message_image.file()?;

        if file.size > 4 * MEBIBYTE {
            Err("the image cannot be larger than 4 MiB.")?;
        }

        let File::File(file) =
            functions::download_file(file.id, 1, 0, 0, true, ctx.client_id).await?;

        ctx.send_typing().await?;

        let result = different_dimension_me::process(
            ctx.bot_state.http_client.clone(),
            &fs::read(file.local.path).unwrap(),
        )
        .await?;

        let media = result.map_err(|err| {
            if err.message == "IMG_ILLEGAL" {
                format!(
                    "Xi Jinping does not approve of this image and has censored it (error {}: {})",
                    err.code, err.message
                )
            } else {
                err.to_string()
            }
        })?;

        let image_url = media.img_urls.into_iter().next().ok_or("the generation failed.")?;
        let response = ctx.bot_state.http_client.get(&image_url).send().await?;
        let image =
            image::load_from_memory_with_format(&response.bytes().await?, ImageFormat::Jpeg)
                .map_err(|err| err.to_string())?;
        let image = crop_result_image(image);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Png).unwrap();

        let formatted_text =
            message_entities::formatted_text(vec!["open full image".text_url(image_url)]);

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
                    caption: Some(formatted_text),
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

fn crop_result_image(mut image: DynamicImage) -> DynamicImage {
    match (image.width(), image.height()) {
        (800, 1257) => image.crop(20, 543, 758, 504),
        (1000, 930) => image.crop(508, 24, 471, 705),
        _ => image,
    }
}
