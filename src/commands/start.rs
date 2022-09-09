use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::utils::Context;

pub struct Start;

#[async_trait]
impl Command for Start {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        ctx.reply_markdown(concat!(
            "Use the /generate command to generate images\\.\n",
            "*Example:* `/generate crayons in a box`"
        ))
        .await?;

        Ok(())
    }
}
