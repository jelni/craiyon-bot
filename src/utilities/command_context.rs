use std::sync::{Arc, Mutex};

use tdlib::enums::{self, ChatAction, InputMessageContent, TextParseMode};
use tdlib::functions;
use tdlib::types::{FormattedText, InputMessageText, Message, TextParseModeMarkdown};

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

    async fn _reply_text(&self, text: FormattedText) -> TdResult<Message> {
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
        self._reply_text(FormattedText { text: text.into(), ..Default::default() }).await
    }

    pub async fn reply_markdown<S: Into<String>>(&self, text: S) -> TdResult<Message> {
        let enums::FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            text.into(),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            self.client_id,
        )
        .await?;

        self._reply_text(formatted_text).await
    }

    async fn _edit_message(&self, message_id: i64, text: FormattedText) -> TdResult<Message> {
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

    #[allow(dead_code)]
    pub async fn edit_message<S: Into<String>>(
        &self,
        message_id: i64,
        text: S,
    ) -> TdResult<Message> {
        self._edit_message(message_id, FormattedText { text: text.into(), ..Default::default() })
            .await
    }

    pub async fn edit_message_markdown<S: Into<String>>(
        &self,
        message_id: i64,
        text: S,
    ) -> TdResult<Message> {
        let enums::FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            text.into(),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            self.client_id,
        )
        .await?;

        self._edit_message(message_id, formatted_text).await
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
