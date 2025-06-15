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
    model: &'static str,
    rate_limit: (usize, i32),
}

impl OpenRouter {
    pub const fn mistral() -> Self {
        Self {
            command_names: &["mistral"],
            description: "ask Mistral Small 3",
            model: "mistralai/mistral-small-24b-instruct-2501",
            rate_limit: (6, 60),
        }
    }

    pub const fn perplexity() -> Self {
        Self {
            command_names: &["perplexity", "sonar"],
            description: "ask Perplexity Sonar",
            model: "perplexity/sonar",
            rate_limit: (2, 120),
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
        RateLimiter::new(self.rate_limit.0, self.rate_limit.1)
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

        if prompt_messages.len() <= 1 {
            return Err(CommandError::Custom("no prompt provided.".into()));
        }

        let response = openai::chat_completion(
            ctx.bot_state.http_client.clone(),
            "https://openrouter.ai/api/v1",
            &env::var("OPENROUTER_API_KEY").unwrap(),
            self.model,
            512,
            &prompt_messages,
        )
        .await?
        .map_err(|err| {
            CommandError::Custom(Cow::Owned(format!("error {}: {}", err.code, err.message)))
        })?;

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
