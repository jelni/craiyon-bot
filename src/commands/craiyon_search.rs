use std::format;
use std::io::BufWriter;

use async_trait::async_trait;
use image::ImageFormat;
use image::imageops::FilterType;
use reqwest::Url;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;

use super::{CommandResult, CommandTrait};
use crate::apis::craiyon;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{ToEntity, ToEntityOwned};
use crate::utilities::text_utils::TruncateWithEllipsis;
use crate::utilities::{api_utils, image_utils, message_entities};

pub struct CraiyonSearch;

#[async_trait]
impl CommandTrait for CraiyonSearch {
    fn command_names(&self) -> &[&str] {
        &["search", "craiyon_search"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("search images generated with 🖍 Craiyon")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(query) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let results = craiyon::search(ctx.bot_state.http_client.clone(), &query)
            .await?
            .into_iter()
            .take(9)
            .collect::<Vec<_>>();
        let urls = results
            .iter()
            .map(|result| {
                Url::parse(&format!("https://pics.craiyon.com/{}", result.image_id)).unwrap()
            })
            .collect();
        let images =
            api_utils::simultaneous_download(ctx.bot_state.http_client.clone(), urls).await?;

        let images = images
            .into_iter()
            .flat_map(|image| image::load_from_memory_with_format(&image, ImageFormat::WebP))
            .map(|image| image.resize_exact(512, 512, FilterType::Lanczos3))
            .collect::<Vec<_>>();

        let image = image_utils::collage(images, (512, 512), 8);
        let mut temp_file = NamedTempFile::new().unwrap();
        image.write_to(&mut BufWriter::new(&mut temp_file), ImageFormat::Png).unwrap();

        let formatted_text = message_entities::formatted_text(
            results
                .into_iter()
                .enumerate()
                .flat_map(|(i, result)| {
                    [
                        "\n".text(),
                        (i + 1).to_string().text_owned(),
                        ". ".text(),
                        result.prompt.truncate_with_ellipsis(128).text_url_owned(format!(
                            "https://pics.craiyon.com/{}",
                            result.image_id
                        )),
                    ]
                })
                .skip(1)
                .collect::<Vec<_>>(),
        );

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
