use std::env;
use std::fmt::Write;

use async_trait::async_trait;
use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::fal::{self, FalResponse};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::rate_limit::RateLimiter;

pub struct Fal;

#[async_trait]
impl CommandTrait for Fal {
    fn command_names(&self) -> &[&str] {
        &["fal", "img", "image"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("generate an image using RealVisXL v4 from fal.ai")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60) // this model costs around $1 / 1200img
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = StringGreedyOrReply::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let response = fal::generate_image(
            ctx.bot_state.http_client.clone(),
            &env::var("FAL_API_KEY").unwrap(),
            "realistic-vision", // model
            "SG161222/RealVisXL_V4.0", // submodel (exact model)
            &prompt,
            String::new(),
        )
        .await?
        .map_err(|err| CommandError::Custom(format!("error {}: {}", err.code, err.message)))?;

        let image_url = response.images.into_iter().next().unwrap();

        ctx.reply_image(image_url, caption).await?;

        Ok(())
    }
}