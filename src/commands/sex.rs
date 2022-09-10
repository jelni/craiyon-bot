use std::error::Error;

use async_trait::async_trait;
use tgbotapi::FileType;

use super::Command;
use crate::utils::Context;

const SEX: [&str; 2] = [
    "CAACAgQAAxkBAAEX8npjHImztCnVUekWoGsQcoqzITtAiAACsQwAAhKVaVMIFeTFdsnn_CkE",
    "CAACAgQAAxkBAAEX9DljHNrRW0S-xydtOOE7n9g4pFEixAACfwsAAhbWcFPn8gyFbHwwoykE",
];

pub struct Sex;

#[async_trait]
impl Command for Sex {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let question_mark = ctx.arguments.as_ref().map_or(false, |a| a.starts_with('?'));
        ctx.send_sticker(FileType::FileID(SEX[usize::from(question_mark)].to_string())).await?;

        Ok(())
    }
}
