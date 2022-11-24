use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tdlib::functions;

use super::{CommandResult, CommandTrait};
use crate::utils::Context;

#[derive(Default)]
pub struct Ping;

#[async_trait]
impl CommandTrait for Ping {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn execute(&self, ctx: Arc<Context>, _: Option<String>) -> CommandResult {
        let start = Instant::now();
        functions::test_network(ctx.client_id).await?;
        let duration = start.elapsed();
        ctx.reply(format!("ping: {}ms", duration.as_millis())).await?;

        Ok(())
    }
}
