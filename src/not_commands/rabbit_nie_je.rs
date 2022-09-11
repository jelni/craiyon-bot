use crate::utils::Context;

#[allow(clippy::unreadable_literal)]
const RABBIT_JE: i64 = -1001722954366;

pub async fn rabbit_nie_je(ctx: Context) {
    if let Some(chat) = &ctx.message.forward_from_chat {
        if chat.id == RABBIT_JE {
            let result = match ctx.delete_message(&ctx.message).await {
                Ok(_) => "Deleted",
                Err(_) => "Couldn't delete",
            };
            log::warn!(
                "{result} a message from in {:?}",
                chat.title.as_deref().unwrap_or_default()
            );
        }
    }
}
