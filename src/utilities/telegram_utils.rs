use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, ReplyMarkup,
};
use tdlib::functions;
use tdlib::types::{
    File, FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message,
    ReplyMarkupInlineKeyboard, UpdateChatMember, User,
};

use super::cache::CompactChat;

pub trait MainUsername {
    fn main_username(&self) -> Option<&String>;
}

impl MainUsername for User {
    fn main_username(&self) -> Option<&String> {
        self.usernames.as_ref()?.active_usernames.first()
    }
}

pub fn donate_markup(name: &str, url: impl Into<String>) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {name}"),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
}

pub fn get_message_text(message: &Message) -> Option<&FormattedText> {
    let formatted_text = match &message.content {
        MessageContent::MessageText(text) => &text.text,
        MessageContent::MessageAnimation(animation) => &animation.caption,
        MessageContent::MessageAudio(audio) => &audio.caption,
        MessageContent::MessageDocument(document) => &document.caption,
        MessageContent::MessagePhoto(photo) => &photo.caption,
        MessageContent::MessageVideo(video) => &video.caption,
        MessageContent::MessageVoiceNote(voice_note) => &voice_note.caption,
        _ => return None,
    };

    Some(formatted_text)
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

    if message.reply_to_message_id == 0 {
        return None;
    }

    let enums::Message::Message(message) =
        functions::get_message(message.reply_in_chat_id, message.reply_to_message_id, client_id)
            .await
            .ok()?;

    get_message_image(&message)
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
