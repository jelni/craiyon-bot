use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileRemote, InputMessageSticker};

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedy};

const SEX: [&str; 2] = [
    "CAACAgQAAxkBAAIHfGOBPouzDkVHO9WAvBrBcMShtX5PAAKxDAACEpVpUwgV5MV2yef8JAQ",
    "CAACAgQAAxkBAAIHe2OBPolUMdfqvn_-38aWQ3bJ0NojAAJ_CwACFtZwU-fyDIVsfDCjJAQ",
];

pub struct Sex;

#[async_trait]
impl CommandTrait for Sex {
    fn command_names(&self) -> &[&str] {
        &["sex", "xes"]
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let argument = Option::<StringGreedy>::convert(ctx, &arguments).await?.0;
        let question_mark = argument.map_or(false, |argument| argument.0.starts_with('?'));

        ctx.reply_custom(
            InputMessageContent::InputMessageSticker(InputMessageSticker {
                sticker: InputFile::Remote(InputFileRemote {
                    id: SEX[usize::from(question_mark)].into(),
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
