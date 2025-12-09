use std::sync::Arc;

use tdlib::enums::{self, ChatAction, InputMessageContent, InputMessageReplyTo};
use tdlib::functions;
use tdlib::types::{
    FormattedText, InputMessageReplyToMessage, InputMessageText, LinkPreviewOptions, Message,
};

use super::bot_state::BotState;
use super::cache::{CompactChat, CompactUser};
use crate::bot::TdResult;

pub struct CommandContext {
    pub client_id: i32,
    pub chat: CompactChat,
    pub user: CompactUser,
    pub message: Message,
    pub bot_state: Arc<BotState>,
}

impl CommandContext {
    pub async fn reply_custom(
        &self,
        message_content: InputMessageContent,
        reply_markup: Option<enums::ReplyMarkup>,
    ) -> TdResult<Message> {
        let enums::Message::Message(message) = functions::send_message(
            self.message.chat_id,
            self.message.topic_id.clone(),
            Some(InputMessageReplyTo::Message(InputMessageReplyToMessage {
                message_id: self.message.id,
                ..Default::default()
            })),
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
                link_preview_options: Some(LinkPreviewOptions {
                    is_disabled: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            None,
        )
        .await
    }

    pub async fn reply_webpage(&self, text: String) -> TdResult<Message> {
        self.reply_custom(
            InputMessageContent::InputMessageText(InputMessageText {
                text: FormattedText { text, ..Default::default() },
                link_preview_options: None,
                ..Default::default()
            }),
            None,
        )
        .await
    }

    pub async fn reply(&self, text: String) -> TdResult<Message> {
        self.reply_formatted_text(FormattedText { text, ..Default::default() }).await
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
                link_preview_options: Some(LinkPreviewOptions {
                    is_disabled: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            self.client_id,
        )
        .await?;

        Ok(message)
    }

    pub async fn edit_message(&self, message_id: i64, text: String) -> TdResult<Message> {
        self.edit_message_formatted_text(message_id, FormattedText { text, ..Default::default() })
            .await
    }

    pub async fn delete_messages(&self, message_ids: Vec<i64>) -> TdResult<()> {
        functions::delete_messages(self.message.chat_id, message_ids, true, self.client_id).await
    }

    pub async fn delete_message(&self, message_id: i64) -> TdResult<()> {
        self.delete_messages(vec![message_id]).await
    }

    pub async fn send_typing(&self) -> TdResult<()> {
        let Some(topic_id) = self.message.topic_id.clone() else {
            return Ok(());
        };

        functions::send_chat_action(
            self.message.chat_id,
            topic_id,
            String::new(),
            Some(ChatAction::Typing),
            self.client_id,
        )
        .await
    }
}
