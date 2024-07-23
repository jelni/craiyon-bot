use std::borrow::Cow;

use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, MessageReplyTo,
    ReplyMarkup,
};
use tdlib::functions;
use tdlib::types::{
    Animation, Audio, Document, File, FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message, Photo, ReplyMarkupInlineKeyboard, UpdateChatMember, User, Video, VoiceNote
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
    Photo(Photo),
    Video(Video),
    Animation(Animation),
    Audio(Audio),
    Voice(VoiceNote),
    Document(Document),
}

impl MessageAttachment {
    pub fn filesize(&self) -> i64 {
        match self {
            MessageAttachment::Photo(photo) => largest_photo(photo).map(|file| file.size).unwrap_or(0),
            MessageAttachment::Video(video) => video.video.size,
            MessageAttachment::Animation(animation) => animation.animation.size,
            MessageAttachment::Audio(audio) => audio.audio.size,
            MessageAttachment::Voice(voice) => voice.voice.size,
            MessageAttachment::Document(document) => document.document.size,
        }
    }

    pub fn file_id(&self) -> i32 {
        match self {
            MessageAttachment::Photo(photo) => largest_photo(photo).map(|file| file.id).unwrap_or(0),
            MessageAttachment::Video(video) => video.video.id,
            MessageAttachment::Animation(animation) => animation.animation.id,
            MessageAttachment::Audio(audio) => audio.audio.id,
            MessageAttachment::Voice(voice) => voice.voice.id,
            MessageAttachment::Document(document) => document.document.id,
        }
    }

    pub fn mime_type(&self) -> Cow<'static, str> {
        match self {
            MessageAttachment::Photo(_) => Cow::Borrowed("image/jpeg"),
            MessageAttachment::Video(video) => Cow::Owned(video.mime_type.clone()),
            MessageAttachment::Animation(_) => Cow::Borrowed("video/mp4"),
            MessageAttachment::Audio(audio) => Cow::Owned(audio.mime_type.clone()),
            MessageAttachment::Voice(voice) => Cow::Owned(voice.mime_type.clone()),
            MessageAttachment::Document(document) => Cow::Owned(document.mime_type.clone()),
        }
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
        MessageContent::MessageDocument(message) => Some(MessageAttachment::Document(message.document.clone())),
        MessageContent::MessagePhoto(message) => Some(MessageAttachment::Photo(message.photo.clone())),
        MessageContent::MessageVideo(message) => Some(MessageAttachment::Video(message.video.clone())),
        MessageContent::MessageAnimation(message) => Some(MessageAttachment::Animation(message.animation.clone())),
        MessageContent::MessageAudio(message) => Some(MessageAttachment::Audio(message.audio.clone())),
        MessageContent::MessageVoiceNote(message) => Some(MessageAttachment::Voice(message.voice_note.clone())),
        _ => None,
    }
}

pub fn get_message_image(content: &MessageContent) -> Option<MessageAttachment> {
    match content {
        MessageContent::MessageDocument(message) => Some(MessageAttachment::Document(message.document.clone())),
        MessageContent::MessagePhoto(message) => Some(MessageAttachment::Photo(message.photo.clone())),
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

pub async fn get_message_or_reply_image(message: &Message, client_id: i32) -> Option<MessageAttachment> {
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
