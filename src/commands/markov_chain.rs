use async_trait::async_trait;
use tdlib::enums::ChatType;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, ToEntity};

pub struct MarkovChain;

#[async_trait]
impl CommandTrait for MarkovChain {
    fn command_names(&self) -> &[&str] {
        &["markov_chain", "markov", "chain"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("generate text based on seen chat messages")
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        if !matches!(ctx.chat.r#type, ChatType::Private(_))
            && !ctx
                .bot_state
                .config
                .lock()
                .unwrap()
                .markov_chain_learning
                .contains(&ctx.message.chat_id)
        {
            ctx.reply_formatted_text(message_entities::formatted_text(vec![
                "a chat admin has to enable Markov chain learning with ".text(),
                "/config markov_chain_learning true".code(),
                ".".text(),
            ]))
            .await?;
            return Ok(());
        }

        let text = ctx.bot_state.markov_chain.lock().unwrap().generate_text(64);
        ctx.reply(text.unwrap_or_else(|| "no text was generated.".into())).await?;

        Ok(())
    }
}
