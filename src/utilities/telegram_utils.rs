use std::borrow::Cow;

use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, MessageReplyTo,
    ReplyMarkup, StickerFormat,
};
use tdlib::functions;
use tdlib::types::{
    File, FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message, Photo,
    ReplyMarkupInlineKeyboard, Sticker, UpdateChatMember, User,
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

pub enum MessageAttachment {
    Image(File),
    Video(File),
    Animation(File),
    Audio(File),
    Voice(File),
    Document(File),
}

impl MessageAttachment {
    pub fn filesize(&self) -> i64 {
        match self {
            MessageAttachment::Image(file) => file.size,
            MessageAttachment::Video(file) => file.size,
            MessageAttachment::Animation(file) => file.size,
            MessageAttachment::Audio(file) => file.size,
            MessageAttachment::Voice(file) => file.size,
            MessageAttachment::Document(file) => file.size,
        }
    }

    pub fn file_id(&self) -> i32 {
        match self {
            MessageAttachment::Image(file) => file.id,
            MessageAttachment::Video(file) => file.id,
            MessageAttachment::Animation(file) => file.id,
            MessageAttachment::Audio(file) => file.id,
            MessageAttachment::Voice(file) => file.id,
            MessageAttachment::Document(file) => file.id,
        }
    }

    pub fn mime_type(&self) -> Cow<'static, str> {
        match self {
            MessageAttachment::Image(_) => Cow::Borrowed("image/jpeg"),
            MessageAttachment::Video(_) => Cow::Borrowed("video/mp4"),
            MessageAttachment::Animation(_) => Cow::Borrowed("video/mp4"),
            MessageAttachment::Audio(_) => Cow::Borrowed("audio/mpeg"),
            MessageAttachment::Voice(_) => Cow::Borrowed("audio/ogg"),
            MessageAttachment::Document(file) => Cow::Borrowed(&file.mime_type),
        }
    }
}

pub struct MessageImage {
    pub file: File,
    pub mime_type: Cow<'static, str>,
}

pub fn donate_markup(name: &str, url: impl Into<String>) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {name}"),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
}

pub const fn get_message_text(content: &MessageContent) -> Option<&FormattedText> {
    let formatted_text = match content {
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

pub fn get_message_attachment(content: &MessageContent) -> Option<MessageAttachment> {
    match content {
        MessageContent::MessageDocument(message) => Some(MessageAttachment::Document(message.document.document.clone())),
        MessageContent::MessagePhoto(message) => largest_photo(&message.photo).map(|file| MessageAttachment::Image(file.clone())),
        MessageContent::MessageVideo(message) => Some(MessageAttachment::Video(message.video.video.clone())),
        MessageContent::MessageAnimation(message) => Some(MessageAttachment::Animation(message.animation.animation.clone())),
        MessageContent::MessageAudio(message) => Some(MessageAttachment::Audio(message.audio.audio.clone())),
        MessageContent::MessageVoiceNote(message) => Some(MessageAttachment::Voice(message.voice_note.voice.clone())),
        _ => None,
    }
}

pub fn get_message_image(content: &MessageContent) -> Option<MessageImage> {
    match content {
        MessageContent::MessageDocument(message) => Some(MessageImage {
            file: message.document.document.clone(),
            mime_type: Cow::Owned(message.document.mime_type.clone()),
        }),
        MessageContent::MessagePhoto(message) => largest_photo(&message.photo).map(|file| {
            MessageImage { file: file.clone(), mime_type: Cow::Borrowed("image/jpeg") }
        }),
        MessageContent::MessageSticker(message) => sticker_image(&message.sticker),
        _ => None,
    }
}

fn largest_photo(photo: &Photo) -> Option<&File> {
    photo
        .sizes
        .iter()
        .rfind(|photo_size| photo_size.photo.local.can_be_downloaded)
        .map(|photo_size| &photo_size.photo)
}

fn sticker_image(sticker: &Sticker) -> Option<MessageImage> {
    (sticker.format == StickerFormat::Webp)
        .then_some(MessageImage { file: sticker.sticker.clone(), mime_type: "image/webp".into() })
}

pub async fn get_message_or_reply_image(message: &Message, client_id: i32) -> Option<MessageImage> {
    if let Some(message_image) = get_message_image(&message.content) {
        return Some(message_image);
    }

    let MessageReplyTo::Message(reply) = message.reply_to.as_ref()? else {
        return None;
    };

    let content = if let Some(content) = reply.content.as_ref() {
        Cow::Borrowed(content)
    } else {
        let enums::Message::Message(message) =
            functions::get_replied_message(message.chat_id, message.id, client_id).await.ok()?;

        Cow::Owned(message.content)
    };

    get_message_image(&content)
}

pub async fn get_message_or_reply_attachment(message: &Message, client_id: i32) -> Option<MessageAttachment> {
    if let Some(attachment) = get_message_attachment(&message.content) {
        return Some(attachment);
    }

    let MessageReplyTo::Message(reply) = message.reply_to.as_ref()? else {
        return None;
    };

    let content = if let Some(content) = reply.content.as_ref() {
        Cow::Borrowed(content)
    } else {
        let enums::Message::Message(message) =
            functions::get_replied_message(message.chat_id, message.id, client_id).await.ok()?;

        Cow::Owned(message.content)
    };

    get_message_attachment(&content)
}

pub fn log_status_update(update: &UpdateChatMember, chat: &CompactChat) {
    if let ChatType::Private(_) = chat.r#type {
        return;
    }

    if update.new_chat_member.status == update.old_chat_member.status {
        return;
    }

    let status = match update.new_chat_member.status {
        ChatMemberStatus::Member => "joined",
        ChatMemberStatus::Left => "left",
        _ => return,
    };

    log::info!("{} {}", status, chat);
}
