use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use super::CommandTrait;
use crate::utils::Context;

#[derive(Default)]
pub struct Ping;

#[async_trait]
impl CommandTrait for Ping {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let start = Instant::now();
        let message = ctx.reply("Measuring…").await?;
        let duration = start.elapsed();
        ctx.edit_message(&message, format!("Ping: {}ms", duration.as_millis())).await?;

        Ok(())
    }
}
