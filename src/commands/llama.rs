use async_trait::async_trait;

use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::openai;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::rate_limit::RateLimiter;

pub struct Llama;

#[async_trait]
impl CommandTrait for Llama {
    fn command_names(&self) -> &[&str] {
        &["llama3", "llama", "groq"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask Llama 3 70B")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 60)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let prompt: Option<StringGreedyOrReply> = Option::<StringGreedyOrReply>::convert(ctx, &arguments).await?.0;
        let prompt = prompt.unwrap().0;
    
        ctx.send_typing().await?;
        
        let token = std::env::var("GROQ_API_KEY").unwrap();
    
        let response = openai::chat_completion(
            ctx.bot_state.http_client.clone(),
            &token,
            "https://api.groq.com/openai/v1",
            "llama3-70b-8192",
            &prompt,
            None,
        )
        .await;
    
        let response = response?;
    
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
