use std::sync::Arc;

use tdlib::enums::{ChatType, MessageContent, MessageSender, TextEntityType, UserType};
use tdlib::types::{Message, MessageSenderUser};

use super::bot_state::BotState;
use super::command_context::CommandContext;
use super::command_manager::CommandInstance;
use super::parsed_command::ParsedCommand;
use super::telegram_utils;
use crate::bot::Bot;

pub enum MessageDestination {
    Command { command: Arc<CommandInstance>, arguments: String, context: CommandContext },
    Dice { message: Message },
    MarkovChain { text: String },
}

pub fn message_destination(
    bot: &Bot,
    bot_state: Arc<BotState>,
    message: Message,
) -> Option<MessageDestination> {
    if message.forward_info.is_some() {
        return None; // ignore forwarded messages
    }

    let MessageSender::User(MessageSenderUser { user_id }) = message.sender_id else {
        return None; // ignore messages not sent by users
    };

    let Some(user) = bot_state.cache.lock().unwrap().get_user(user_id) else {
        log::warn!("user {user_id} not found in cache");
        return None; // ignore users not in cache
    };

    let UserType::Regular = user.r#type else {
        return None; // ignore bots
    };

    let Some(chat) = bot_state.cache.lock().unwrap().get_chat(message.chat_id) else {
        log::warn!("chat {} not found in cache", message.chat_id);
        return None; // ignore chats not in cache
    };

    if let MessageContent::MessageDice(_) = message.content {
        return Some(MessageDestination::Dice { message });
    }

    let Some(text) = telegram_utils::get_message_text(&message) else {
        return None; // ignore messages without text
    };

    if let Some(parsed_command) = ParsedCommand::parse(text) {
        if let Some(bot_username) = &parsed_command.bot_username {
            let Some(me) = bot.get_me() else {
                log::warn!("client user not cached");
                return None; // return if the client user is not cached
            };

            let username = me.username?; // return if the client user has no username

            if username.to_ascii_lowercase() != *bot_username.to_ascii_lowercase() {
                return None; // ignore commands sent to other bots
            }
        }

        let Some(command) = bot.get_command(&parsed_command.name) else {
            return None; // ignore nonexistent commands
        };

        Some(MessageDestination::Command {
            command,
            arguments: parsed_command.arguments,
            context: CommandContext { client_id: bot.client_id, chat, user, message, bot_state },
        })
    } else {
        let (ChatType::BasicGroup(_) | ChatType::Supergroup(_)) = chat.r#type else {
            return None; // ignore messages not in a group
        };

        if text.entities.iter().any(|entity| {
            matches!(
                entity.r#type,
                TextEntityType::Mention
                    | TextEntityType::Url
                    | TextEntityType::EmailAddress
                    | TextEntityType::PhoneNumber
                    | TextEntityType::BankCardNumber
                    | TextEntityType::MentionName(_)
            )
        }) {
            return None; // ignore messages with sensitive data
        }

        if !bot_state.config.lock().unwrap().markov_chain_learning.contains(&message.chat_id) {
            return None; // ignore messages if Markov chain learning is disabled
        }

        Some(MessageDestination::MarkovChain { text: text.text.clone() })
    }
}
