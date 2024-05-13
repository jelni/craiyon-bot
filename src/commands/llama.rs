use std::fmt::Write;

use async_trait::async_trait;

use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::makersuite::Part;
use crate::apis::openai;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::rate_limit::RateLimiter;

pub struct Llama;

#[async_trait]
impl CommandTrait for Llama {
    fn command_names(&self) -> &[&str] {
        &["llama", "groq", "llama3"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask Llama 3 70B")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let prompt = Option::<StringGreedyOrReply>::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let model = "llama3-70b-8192";
        let token = std::env::var("GROQ_API_KEY").unwrap();
        let mut parts = Vec::new();

        if let Some(prompt) = prompt {
            parts.push(Part::Text(prompt.0));
        }

        if parts.is_empty() {
            return Err(CommandError::Custom("no prompt provided.".into()));
        }

        // Convert text parts into a single string
        let mut parts_str = String::new();
        for part in parts {
            if let Part::Text(text) = part {
                writeln!(parts_str, "{text}").unwrap();
            }
        }

        let http_client = ctx.bot_state.http_client.clone();

        let response = openai::generate_content(
            "https://api.groq.com/openai/v1",
            model,
            &token,
            http_client,
            &parts_str,
        )
        .await;

        if let Err(e) = response {
            return Err(CommandError::Custom(e.to_string()));
        }
        let response = response.unwrap();

        match response {
            Ok(response) => {
                let text = response.choices.into_iter().next().unwrap().message.content;

                let enums::FormattedText::FormattedText(formatted_text) =
                    functions::parse_markdown(
                        FormattedText { text, ..Default::default() },
                        ctx.client_id,
                    )
                    .await?;

                let unsent_message = ctx.reply_formatted_text(formatted_text).await?;
                let _message =
                    ctx.bot_state.message_queue.wait_for_message(unsent_message.id).await?;

                Ok(())
            }
            Err(e) => Err(CommandError::Custom(e.message)),
        }
    }
}
