use std::sync::Arc;

use async_trait::async_trait;
use tgbotapi::FileType;

use super::{CommandResult, CommandTrait};
use crate::utils::Context;

const SEX: [&str; 2] = [
    "CAACAgQAAxkBAAEX8npjHImztCnVUekWoGsQcoqzITtAiAACsQwAAhKVaVMIFeTFdsnn_CkE",
    "CAACAgQAAxkBAAEX9DljHNrRW0S-xydtOOE7n9g4pFEixAACfwsAAhbWcFPn8gyFbHwwoykE",
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
        ctx.send_sticker(FileType::FileID(SEX[usize::from(question_mark)].to_string())).await?;

        Ok(())
    }
}
