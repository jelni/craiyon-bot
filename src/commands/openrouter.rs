use std::borrow::Cow;
use std::env;
use std::fmt::Write;

use async_trait::async_trait;
use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::gemini::SYSTEM_INSTRUCTION;
use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::openai::{self, Message};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, ReplyChain};
use crate::utilities::rate_limit::RateLimiter;

pub struct OpenRouter {
    command_names: &'static [&'static str],
    description: &'static str,
    model_name: &'static str,
    ratelimit: RateLimiter<i64>,
}

impl OpenRouter {
    pub fn mistral() -> Self {
        Self {
            command_names: &["mistral"],
            description: "generate text using mistral small 3",
            model_name: "mistralai/mistral-small-24b-instruct-2501",
            ratelimit: RateLimiter::new(6, 60),
        }
    }
    pub fn sonar() -> Self {
        Self {
            command_names: &["sonar", "online"],
            description: "search the web using perplexity sonar",
            model_name: "perplexity/sonar",
            ratelimit: RateLimiter::new(1, 120),
        }
    }
}

#[async_trait]
impl CommandTrait for OpenRouter {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        self.ratelimit.clone()
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let ReplyChain(messages) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let mut prompt_messages =
            vec![Message { content: Cow::Borrowed(SYSTEM_INSTRUCTION), role: "system" }];

        prompt_messages.extend(messages.into_iter().filter_map(|message| {
            message.text.map(|text| Message {
                role: if message.bot_author { "assistant" } else { "user" },
                content: Cow::Owned(text),
            })
        }));

        let response = openai::chat_completion(
            ctx.bot_state.http_client.clone(),
            "https://openrouter.ai/api/v1",
            &env::var("OPENROUTER_API_KEY").unwrap(),
            self.model_name,
            2048,
            0.5,
            &prompt_messages,
        )
        .await?
        .map_err(|err| CommandError::Custom(format!("error {}: {}", err.code, err.message)))?;

        let choice = response.choices.into_iter().next().unwrap();
        let mut text = choice.message.content;

        if choice.finish_reason != "stop" {
            write!(text, " [{}]", choice.finish_reason).unwrap();
        }

        let formatted_text = if text.trim().is_empty() {
            FormattedText { text: "[no text generated]".into(), ..Default::default() }
        } else {
            let enums::FormattedText::FormattedText(formatted_text) = functions::parse_markdown(
                FormattedText { text, ..Default::default() },
                ctx.client_id,
            )
            .await?;

            formatted_text
        };

        ctx.reply_formatted_text(formatted_text).await?;

        Ok(())
    }
}
