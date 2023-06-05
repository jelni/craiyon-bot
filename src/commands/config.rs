use async_trait::async_trait;
use tdlib::enums::ChatType;

use super::{CommandError, CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::ConvertArgument;
use crate::utilities::message_entities::{self, Entity, ToEntity};

const MARKOV_CHAIN_LEARNING: &str = "markov_chain_learning";
const OPTIONS: [&str; 1] = [MARKOV_CHAIN_LEARNING];

pub struct Config;

#[async_trait]
impl CommandTrait for Config {
    fn command_names(&self) -> &[&str] {
        &["config", "set"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("configure bot options")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let Ok((mut option, rest)) = String::convert(ctx, &arguments).await else {
            let mut entities = vec!["list of available options:\n".text()];
            entities.extend(option_names());
            ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;
            return Ok(());
        };

        option.make_ascii_lowercase();

        if option == MARKOV_CHAIN_LEARNING {
            chat_group_guard(ctx)?;

            let value = bool::convert(ctx, rest).await?.0;
            if value {
                let changed = ctx
                    .bot_state
                    .config
                    .lock()
                    .unwrap()
                    .markov_chain_learning
                    .insert(ctx.message.chat_id);
                if changed {
                    ctx.reply("Markov chain will now learn from chat messages.").await?;
                } else {
                    ctx.reply("Markov chain learning was already enabled.").await?;
                }
            } else {
                let changed = ctx
                    .bot_state
                    .config
                    .lock()
                    .unwrap()
                    .markov_chain_learning
                    .remove(&ctx.message.chat_id);
                if changed {
                    ctx.reply("Markov chain won't learn from chat messages anymore.").await?;
                } else {
                    ctx.reply("Markov chain learning was already disabled.").await?;
                }
            };
        } else {
            let mut entities = vec!["unknown option name. available options include:\n".text()];
            entities.extend(option_names());

            Err(CommandError::CustomFormattedText(message_entities::formatted_text(entities)))?;
        }

        Ok(())
    }
}

fn chat_group_guard(ctx: &CommandContext) -> CommandResult {
    let (ChatType::BasicGroup(_) | ChatType::Supergroup(_)) = ctx.chat.r#type else {
        return Err(CommandError::Custom("this option can be only set in groups.".into()));
    };

    Ok(())
}

fn option_names() -> impl Iterator<Item = Entity<'static>> {
    OPTIONS.into_iter().flat_map(|option| [",\n".text(), option.code()]).skip(1)
}
