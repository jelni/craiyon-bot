use std::borrow::Cow;

use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, MessageReplyTo, ReplyMarkup, StickerFormat
};
use tdlib::functions;
use tdlib::types::{
    Animation, Audio, Document, File, FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message, Photo, ReplyMarkupInlineKeyboard, Sticker, UpdateChatMember, User, Video, VideoNote, VoiceNote
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
    Animation(Animation),
    Audio(Audio),
    Document(Document),
    Photo(Photo),
    Sticker(Sticker),
    Video(Video),
    VideoNote(VideoNote),
    VoiceNote(VoiceNote),

}

impl MessageAttachment {
    pub fn file(&self) -> &File {
        match self {
            MessageAttachment::Animation(animation) => &animation.animation,
            MessageAttachment::Audio(audio) => &audio.audio,
            MessageAttachment::Document(document) => &document.document,
            MessageAttachment::Photo(photo) => largest_photo(photo).unwrap(),
            MessageAttachment::Sticker(sticker) => &sticker.sticker,
            MessageAttachment::Video(video) => &video.video,
            MessageAttachment::VideoNote(video_note) => &video_note.video,
            MessageAttachment::VoiceNote(voice_note) => &voice_note.voice,
    }}

    pub fn mime_type(&self) -> Cow<'static, str> {
        match self {
            MessageAttachment::Animation(animation) => Cow::Owned(animation.mime_type.clone()),
            MessageAttachment::Audio(audio) => Cow::Owned(audio.mime_type.clone()),
            MessageAttachment::Document(document) => Cow::Owned(document.mime_type.clone()),
            MessageAttachment::Photo(_) => {
                Cow::Owned("image/jpeg".to_string())
            }
            MessageAttachment::Sticker(sticker) => get_sticker_format(sticker).clone(),
            MessageAttachment::Video(video) => Cow::Owned(video.mime_type.clone()),
            MessageAttachment::VideoNote(_) => Cow::Owned("video/mp4".to_string()),
            MessageAttachment::VoiceNote(voice_note) => Cow::Owned(voice_note.mime_type.clone()),
        }
    }
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
        MessageContent::MessageVideoNote(message) => Some(MessageAttachment::VideoNote(message.video_note.clone())),
        MessageContent::MessageVoiceNote(message) => Some(MessageAttachment::VoiceNote(message.voice_note.clone())),
        MessageContent::MessageSticker(message) => Some(MessageAttachment::Sticker(message.sticker.clone())),
        _ => None,
    }
}

pub fn get_sticker_format(sticker: &Sticker) -> Cow<'static, str> {
    match sticker.format {
        StickerFormat::Webp => Cow::Borrowed("image/webp"),
        StickerFormat::Tgs => Cow::Borrowed("application/x-tgsticker"),
        StickerFormat::Webm => Cow::Borrowed("video/webm"),
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
