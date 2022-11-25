use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileRemote, InputMessageSticker};

use super::{CommandResult, CommandTrait};
use crate::utils::Context;

const SEX: [&str; 2] = [
    "CAACAgQAAxkBAAIHfGOBPouzDkVHO9WAvBrBcMShtX5PAAKxDAACEpVpUwgV5MV2yef8JAQ",
    "CAACAgQAAxkBAAIHe2OBPolUMdfqvn_-38aWQ3bJ0NojAAJ_CwACFtZwU-fyDIVsfDCjJAQ",
];

#[derive(Default)]
pub struct Sex;

#[async_trait]
impl CommandTrait for Sex {
    fn name(&self) -> &'static str {
        "sex"
    }

    fn aliases(&self) -> &[&str] {
        &["xes"]
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let question_mark = arguments.map_or(false, |a| a.starts_with('?'));
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
