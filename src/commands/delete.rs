use async_trait::async_trait;
use tdlib::enums::MessageReplyTo;
use tdlib::types::MessageReplyToMessage;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;

#[allow(clippy::unreadable_literal)]
const OWNER_ID: i64 = 807128293;

pub struct Delete;

#[async_trait]
impl CommandTrait for Delete {
    fn command_names(&self) -> &[&str] {
        &["delete", "del"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        if ctx.user.id != OWNER_ID {
            return Ok(());
        }

        let Some(&MessageReplyTo::Message(MessageReplyToMessage { message_id, .. })) =
            ctx.message.reply_to.as_ref()
        else {
            return Ok(());
        };

        ctx.delete_message(message_id).await.ok();

        Ok(())
    }
}
