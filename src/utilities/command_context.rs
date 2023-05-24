use std::sync::{Arc, Mutex};

use tdlib::enums::{self, ChatAction, InputMessageContent};
use tdlib::functions;
use tdlib::types::{FormattedText, InputMessageText, Message};

use super::cache::{CompactChat, CompactUser};
use super::message_queue::MessageQueue;
use super::rate_limit::RateLimits;
use crate::bot::TdResult;

pub struct CommandContext {
    pub chat: CompactChat,
    pub user: CompactUser,
    pub message: Message,
    pub client_id: i32,
    pub rate_limits: Arc<Mutex<RateLimits>>,
    pub message_queue: Arc<MessageQueue>,
    pub http_client: reqwest::Client,
}

impl CommandContext {
    pub async fn reply_custom(
        &self,
        message_content: InputMessageContent,
        reply_markup: Option<enums::ReplyMarkup>,
    ) -> TdResult<Message> {
        let enums::Message::Message(message) = functions::send_message(
            self.message.chat_id,
            self.message.message_thread_id,
            self.message.id,
            None,
            reply_markup,
            message_content,
            self.client_id,
        )
        .await?;

        Ok(message)
    }

    pub async fn reply_formatted_text(&self, text: FormattedText) -> TdResult<Message> {
        self.reply_custom(
            InputMessageContent::InputMessageText(InputMessageText {
                text,
                disable_web_page_preview: true,
                ..Default::default()
            }),
            None,
        )
        .await
    }

    pub async fn reply<S: Into<String>>(&self, text: S) -> TdResult<Message> {
        self.reply_formatted_text(FormattedText { text: text.into(), ..Default::default() }).await
    }

    pub async fn edit_message_formatted_text(
        &self,
        message_id: i64,
        text: FormattedText,
    ) -> TdResult<Message> {
        let enums::Message::Message(message) = functions::edit_message_text(
            self.message.chat_id,
            message_id,
            None,
            InputMessageContent::InputMessageText(InputMessageText {
                text,
                disable_web_page_preview: true,
                ..Default::default()
            }),
            self.client_id,
        )
        .await?;

        Ok(message)
    }

    pub async fn edit_message<S: Into<String>>(
        &self,
        message_id: i64,
        text: S,
    ) -> TdResult<Message> {
        self.edit_message_formatted_text(
            message_id,
            FormattedText { text: text.into(), ..Default::default() },
        )
        .await
    }

    pub async fn delete_messages(&self, message_ids: Vec<i64>) -> TdResult<()> {
        functions::delete_messages(self.message.chat_id, message_ids, true, self.client_id).await
    }

    pub async fn delete_message(&self, message_id: i64) -> TdResult<()> {
        self.delete_messages(vec![message_id]).await
    }

    pub async fn send_typing(&self) -> TdResult<()> {
        functions::send_chat_action(
            self.message.chat_id,
            self.message.message_thread_id,
            Some(ChatAction::Typing),
            self.client_id,
        )
        .await
    }
}
