use std::error::Error;

use async_trait::async_trait;
use tgbotapi::FileType;

use super::Command;
use crate::utils::Context;

const SEX: &str = "CAACAgQAAxkBAAEX8npjHImztCnVUekWoGsQcoqzITtAiAACsQwAAhKVaVMIFeTFdsnn_CkE";

pub struct Sex;

#[async_trait]
impl Command for Sex {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        ctx.send_sticker(FileType::FileID(SEX.to_string())).await?;

        Ok(())
    }
}
