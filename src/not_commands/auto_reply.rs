use std::sync::Arc;

use crate::utils::{Context, DisplayUser};

pub async fn auto_reply(ctx: Arc<Context>) {
    let text = match &ctx.message.text {
        Some(text) => text.to_lowercase(),
        None => return,
    };

    let words = text.split_ascii_whitespace().collect::<Vec<_>>();

    let reply = if text.contains("tomasz fryta") {
        "real madryt"
    } else if words.contains(&"obejrz") {
        "obejrzyj"
    } else if text.contains("zapytaj mu") {
        "zapytaj go"
    } else if text.contains("spytaj mu") {
        "spytaj go"
    } else if words.contains(&"poszłem") || words.contains(&"poszlem") {
        "poszedłem"
    } else {
        return;
    };

    if ctx
        .ratelimits
        .write()
        .unwrap()
        .auto_reply
        .update_rate_limit(ctx.user.id, ctx.message.date)
        .is_some()
    {
        log::info!("auto reply ratelimit exceeded by {} ({reply:?})", ctx.user.format_name());
        return;
    }

    ctx.reply(reply).await.ok();
}
