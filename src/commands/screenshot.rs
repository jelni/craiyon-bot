use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Url;
use tgbotapi::FileType;
use url::ParseError;

use super::CommandTrait;
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
        &["ss", "webimage", "webscreenshot"]
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 120)
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("URL to screenshot").await;
            return Ok(());
        };

        let url = match Url::parse(&url) {
            Err(ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("http://{url}")),
            url => url,
        };

        let url = match url {
            Ok(url) => url,
            Err(err) => {
                ctx.reply(err.to_string()).await?;
                return Ok(());
            }
        };

        let data = match microlink::screenshot(ctx.http_client.clone(), url).await? {
            Ok(data) => data,
            Err(err) => {
                ctx.reply_markdown(format!(
                    "[{}]({}): {}",
                    escape_markdown(err.code),
                    err.more,
                    escape_markdown(err.message)
                ))
                .await?;
                return Ok(());
            }
        };

        ctx.api
            .make_request(&SendPhoto {
                chat_id: ctx.message.chat_id(),
                photo: FileType::Url(data.screenshot.url),
                caption: Some(data.title),
                reply_to_message_id: Some(ctx.message.message_id),
                allow_sending_without_reply: Some(true),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
