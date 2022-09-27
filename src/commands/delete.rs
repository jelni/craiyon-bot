use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::utils::Context;

#[allow(clippy::unreadable_literal)]
const OWNER_ID: i64 = 807128293;

#[derive(Default)]
pub struct Delete;

#[async_trait]
impl CommandTrait for Delete {
    fn name(&self) -> &str {
        "delete"
    }

    fn aliases(&self) -> &[&str] {
        &["del"]
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if ctx.user.id != OWNER_ID {
            return Ok(());
        }

        if let Some(message) = &ctx.message.reply_to_message {
            ctx.delete_message(message).await.ok();
        }

        Ok(())
    }
}
