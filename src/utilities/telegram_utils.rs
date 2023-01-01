use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, ReplyMarkup,
};
use tdlib::functions;
use tdlib::types::{
    File, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message, ReplyMarkupInlineKeyboard,
    UpdateChatMember, User,
};

use super::cache::CompactChat;

pub trait MainUsername {
    fn main_username(&self) -> Option<&str>;
}

impl MainUsername for User {
    fn main_username(&self) -> Option<&str> {
        Some(self.usernames.as_ref()?.active_usernames.first()?.as_str())
    }
}

pub fn donate_markup<N: AsRef<str>, U: Into<String>>(name: N, url: U) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {}", name.as_ref()),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
}

pub fn get_message_image(message: &Message) -> Option<File> {
    match &message.content {
        MessageContent::MessageDocument(document) => Some(document.document.document.clone()),
        MessageContent::MessagePhoto(photo) => photo
            .photo
            .sizes
            .iter()
            .rev()
            .find(|photo_size| photo_size.photo.local.can_be_downloaded)
            .map(|photo_size| photo_size.photo.clone()),
        _ => None,
    }
}

pub async fn get_message_or_reply_image(message: &Message, client_id: i32) -> Option<File> {
    if let Some(file) = get_message_image(message) {
        return Some(file);
    }

    if message.reply_to_message_id != 0 {
        let enums::Message::Message(message) = functions::get_message(
            message.reply_in_chat_id,
            message.reply_to_message_id,
            client_id,
        )
        .await
        .ok()?;
        return get_message_image(&message);
    }

    None
}

pub fn log_status_update(update: UpdateChatMember, chat: &CompactChat) {
    if let ChatType::Private(_) = chat.r#type {
        return;
    }

    let new_status = update.new_chat_member.status;
    if new_status == update.old_chat_member.status {
        return;
    }

    let status = match new_status {
        ChatMemberStatus::Member => "joined",
        ChatMemberStatus::Left => "left",
        _ => return,
    };

    log::info!("{} {}", status, chat);
}
