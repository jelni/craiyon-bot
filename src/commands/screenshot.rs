use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Url;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{FormattedText, InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;
use url::ParseError;

use super::CommandError::{CustomMarkdownError, MissingArgument};
use super::{CommandResult, CommandTrait};
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

        let screenshot = ctx
            .http_client
            .get(data.screenshot.url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&screenshot).unwrap();

        let message = ctx
            .reply_custom(
                InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                    photo: InputFile::Local(InputFileLocal {
                        path: temp_file.path().to_str().unwrap().into(),
                    }),
                    thumbnail: None,
                    added_sticker_file_ids: Vec::new(),
                    width: 0,
                    height: 0,
                    caption: data.title.map(|t| FormattedText { text: t, ..Default::default() }),
                    ttl: 0,
                }),
                None,
            )
            .await?;

        ctx.message_queue.wait_for_message(message.id).await?;
        temp_file.close().unwrap();

        Ok(())
    }
}
