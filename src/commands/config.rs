use async_trait::async_trait;
use tdlib::enums::{ChatMemberStatus, ChatType};

use super::{CommandError, CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::ConvertArgument;
use crate::utilities::message_entities::{self, Entity, ToEntity};

const MARKOV_CHAIN_LEARNING: &str = "markov_chain_learning";
const SETTINGS: [&str; 1] = [MARKOV_CHAIN_LEARNING];

pub struct Config;

#[async_trait]
impl CommandTrait for Config {
    fn command_names(&self) -> &[&str] {
        &["config", "set"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("configure bot settings")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let Ok((mut setting, rest)) = String::convert(ctx, &arguments).await else {
            let mut entities = vec!["list of available settings:\n".text()];
            entities.extend(setting_names());
            ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;
            return Ok(());
        };

        setting.make_ascii_lowercase();

        if setting == MARKOV_CHAIN_LEARNING {
            chat_group_guard(ctx)?;
            chat_admin_guard(ctx).await?;

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
                    ctx.reply("Markov chain will now learn from chat messages.".into()).await?;
                } else {
                    ctx.reply("Markov chain learning was already enabled.".into()).await?;
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
                    ctx.reply("Markov chain won't learn from chat messages anymore.".into())
                        .await?;
                } else {
                    ctx.reply("Markov chain learning was already disabled.".into()).await?;
                }
            };
        } else {
            let mut entities = vec!["unknown setting name. available settings include:\n".text()];
            entities.extend(setting_names());

            Err(CommandError::CustomFormattedText(message_entities::formatted_text(entities)))?;
        }

        Ok(())
    }
}

fn chat_group_guard(ctx: &CommandContext) -> CommandResult {
    let (ChatType::BasicGroup(_) | ChatType::Supergroup(_)) = ctx.chat.r#type else {
        return Err("this setting can be only set in groups.".into());
    };

    Ok(())
}

async fn chat_admin_guard(ctx: &CommandContext) -> CommandResult {
    let status =
        ctx.bot_state.get_member_status(ctx.message.chat_id, ctx.user.id, ctx.client_id).await?;

    if !match status {
        ChatMemberStatus::Creator(_) => true,
        ChatMemberStatus::Administrator(status) => status.rights.can_change_info,
        _ => false,
    } {
        return Err("this setting requires the Change Group Info permission.".into());
    }

    Ok(())
}

fn setting_names() -> impl Iterator<Item = Entity<'static>> {
    SETTINGS.into_iter().flat_map(|setting| [",\n".text(), setting.code()]).skip(1)
}
