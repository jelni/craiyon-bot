use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::Command;
use crate::utils::Context;

#[derive(Default)]
pub struct Start;

#[async_trait]
impl Command for Start {
    fn name(&self) -> &str {
        "start"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        ctx.reply_markdown(concat!(
            "Use the /generate command to generate images\\.\n",
            "*Example:* `/generate crayons in a box`"
        ))
        .await?;

        Ok(())
    }
}
