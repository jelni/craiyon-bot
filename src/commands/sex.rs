use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent, StickerSet};
use tdlib::functions;
use tdlib::types::{InputFileRemote, InputMessageSticker};

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedy};

pub struct Sex;

#[async_trait]
impl CommandTrait for Sex {
    fn command_names(&self) -> &[&str] {
        &["sex", "xes"]
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let argument = Option::<StringGreedy>::convert(ctx, &arguments).await?.0;
        let question_mark = argument.is_some_and(|argument| argument.0.starts_with('?'));
        let StickerSet::StickerSet(mut sticker_set) =
            functions::search_sticker_set("fratik_sex".into(), false, ctx.client_id).await?;

        ctx.reply_custom(
            InputMessageContent::InputMessageSticker(InputMessageSticker {
                sticker: InputFile::Remote(InputFileRemote {
                    id: sticker_set.stickers.swap_remove((!question_mark).into()).sticker.remote.id,
                }),
                thumbnail: None,
                width: 0,
                height: 0,
                emoji: String::new(),
            }),
            None,
        )
        .await?;

        Ok(())
    }
}
