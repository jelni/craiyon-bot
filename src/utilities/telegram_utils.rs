use std::borrow::Cow;

use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, MessageReplyTo,
    ReplyMarkup, StickerFormat,
};
use tdlib::functions;
use tdlib::types::{
    File, FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message,
    MessageReplyToMessage, Photo, ReplyMarkupInlineKeyboard, Sticker, UpdateChatMember, User,
    WebPage,
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

pub const fn get_message_text(message: &Message) -> Option<&FormattedText> {
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

pub fn get_message_image(message: &Message) -> Option<MessageImage> {
    match &message.content {
        MessageContent::MessageText(message) => message.web_page.as_ref().and_then(web_page_image),
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

fn web_page_image(web_page: &WebPage) -> Option<MessageImage> {
    if let Some(photo) = &web_page.photo {
        if let Some(file) = largest_photo(photo) {
            return Some(MessageImage {
                file: file.clone(),
                mime_type: Cow::Borrowed("image/jpeg"),
            });
        }
    }

    if let Some(document) = &web_page.document {
        return Some(MessageImage {
            file: document.document.clone(),
            mime_type: Cow::Owned(document.mime_type.clone()),
        });
    }

    if let Some(sticker) = &web_page.sticker {
        if sticker.format == StickerFormat::Webp {
            return Some(MessageImage {
                file: sticker.sticker.clone(),
                mime_type: Cow::Borrowed("image/webp"),
            });
        }
    }

    None
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
    if let Some(message_image) = get_message_image(message) {
        return Some(message_image);
    }

    let &MessageReplyTo::Message(MessageReplyToMessage { chat_id, message_id, .. }) =
        message.reply_to.as_ref()?
    else {
        return None;
    };

    let enums::Message::Message(message) =
        functions::get_message(chat_id, message_id, client_id).await.ok()?;

    get_message_image(&message)
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
