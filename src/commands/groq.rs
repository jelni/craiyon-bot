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

pub struct Groq {
    command_names: &'static [&'static str],
    description: &'static str,
    model_name: &'static str,
}

impl Groq {
    pub const fn llama() -> Self {
        Self {
            command_names: &["llama"],
            description: "generate text using llama 3.3",
            model_name: "llama-3.3-70b-specdec",
        }
    }
    pub const fn thinker() -> Self {
        Self {
            command_names: &["thinker", "r1"],
            description: "generate text using deepseek r1 (70b distilled)",
            model_name: "deepseek-r1-distill-llama-70b",
        }
    }
}

#[async_trait]
impl CommandTrait for Groq {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(6, 60)
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
            "https://api.groq.com/openai/v1",
            &env::var("GROQ_API_KEY").unwrap(),
            self.model_name,
            &prompt_messages,
        )
        .await?
        .map_err(|err| CommandError::Custom(format!("error {}: {}", err.code, err.message)))?;

        let choice = response.choices.into_iter().next().unwrap();
        let mut text = choice.message.content;

        if self.model_name == "deepseek-r1-distill-llama-70b" {
            while let Some(start) = text.find("<think>") {
                if let Some(end) = text[start..].find("</think>") {
                    let removed_len = end + 8;
                    text.replace_range(start..start + end + 8, "");
                    text = text.trim().to_string();
                    // prefix with newline if not already present
                    text.insert_str(0, &format!("[{removed_len} thinking chars hidden]\n\n"));
                } else {
                    break;
                }
            }
        }

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
