use async_trait::async_trait;
use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::{CommandResult, CommandTrait};
use crate::apis::google_palm;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::rate_limit::RateLimiter;

pub struct GooglePalm;

#[async_trait]
impl CommandTrait for GooglePalm {
    fn command_names(&self) -> &[&str] {
        &["google_palm", "palm"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask Google PaLM")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 45)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let response =
            google_palm::generate_text(ctx.http_client.clone(), &(prompt + "\n"), 256).await?;

        let text = match response {
            Ok(response) => {
                if let Some(filters) = response.filters {
                    let reasons = filters
                        .into_iter()
                        .map(|filter| {
                            if let Some(message) = filter.message {
                                format!("{}: {message}", filter.reason)
                            } else {
                                filter.reason
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    ctx.reply(format!("request filtered by Google: {reasons}.",)).await?;
                    return Ok(());
                }

                response.candidates.unwrap().into_iter().next().unwrap().output
            }
            Err(response) => {
                ctx.reply(format!("error {}: {}", response.error.code, response.error.message))
                    .await?;
                return Ok(());
            }
        };

        if text.is_empty() {
            ctx.reply("no text was generated.").await?;
        }

        let enums::FormattedText::FormattedText(formatted_text) =
            functions::parse_markdown(FormattedText { text, ..Default::default() }, ctx.client_id)
                .await?;

        ctx.reply_formatted_text(formatted_text).await?;

        Ok(())
    }
}
