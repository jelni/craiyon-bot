use std::io::Write;

use async_trait::async_trait;
use reqwest::Url;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{FormattedText, InputFileLocal, InputMessagePhoto};
use tempfile::NamedTempFile;
use url::ParseError;

use super::CommandError::CustomMarkdownError;
use super::{CommandResult, CommandTrait};
use crate::apis::microlink;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedy;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::text_utils::EscapeMarkdown;

pub struct Screenshot;

#[async_trait]
impl CommandTrait for Screenshot {
    fn command_names(&self) -> &[&str] {
        &["screenshot", "ss", "webimg", "webimage", "webscreenshot"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("screenshot a webpage")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 120)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedy(url) = ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        let url = match Url::parse(&url) {
            Err(ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("http://{url}")),
            url => url,
        };

        ctx.send_typing().await?;

        let data =
            microlink::screenshot(ctx.http_client.clone(), url.map_err(|err| err.to_string())?)
                .await?
                .map_err(|err| {
                    CustomMarkdownError(format!(
                        "[{}]({}): {}",
                        EscapeMarkdown(&err.code),
                        err.more,
                        EscapeMarkdown(&err.message)
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
                    self_destruct_time: 0,
                    has_spoiler: false,
                }),
                None,
            )
            .await?;

        ctx.message_queue.wait_for_message(message.id).await?;
        temp_file.close().unwrap();

        Ok(())
    }
}
