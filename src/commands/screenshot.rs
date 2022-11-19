use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Url;
use tgbotapi::FileType;
use url::ParseError;

use super::CommandError::{CustomMarkdownError, MissingArgument};
use super::{CommandResult, CommandTrait};
use crate::api_methods::SendPhoto;
use crate::apis::microlink;
use crate::ratelimit::RateLimiter;
use crate::utils::{escape_markdown, Context};

#[derive(Default)]
pub struct Screenshot;

#[async_trait]
impl CommandTrait for Screenshot {
    fn name(&self) -> &'static str {
        "screenshot"
    }

    fn aliases(&self) -> &[&str] {
        &["ss", "webimg", "webimage", "webscreenshot"]
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 120)
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let url = arguments.ok_or(MissingArgument("URL to screenshot"))?;

        let url = match Url::parse(&url) {
            Err(ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("http://{url}")),
            url => url,
        };

        let data =
            microlink::screenshot(ctx.http_client.clone(), url.map_err(|err| err.to_string())?)
                .await?
                .map_err(|err| {
                    CustomMarkdownError(format!(
                        "[{}]({}): {}",
                        escape_markdown(err.code),
                        err.more,
                        escape_markdown(err.message)
                    ))
                })?;

        ctx.api
            .make_request(&SendPhoto {
                chat_id: ctx.message.chat_id(),
                photo: FileType::Url(data.screenshot.url),
                caption: data.title,
                reply_to_message_id: Some(ctx.message.message_id),
                allow_sending_without_reply: Some(true),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
