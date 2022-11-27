use std::sync::Arc;

use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utils::Context;

#[allow(clippy::unreadable_literal)]
const OWNER_ID: i64 = 807128293;

#[derive(Default)]
pub struct Delete;

#[async_trait]
impl CommandTrait for Delete {
    fn command_names(&self) -> &[&str] {
        &["delete", "del"]
    }

    async fn execute(&self, ctx: Arc<Context>, _: Option<String>) -> CommandResult {
        if ctx.user.id != OWNER_ID {
            return Ok(());
        }

        let reply_to_message_id = ctx.message.reply_to_message_id;
        if reply_to_message_id != 0 {
            ctx.delete_message(reply_to_message_id).await.ok();
        }

        Ok(())
    }
}
