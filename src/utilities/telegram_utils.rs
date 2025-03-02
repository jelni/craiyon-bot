use std::borrow::Cow;

use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, MessageReplyTo,
    ReplyMarkup, StickerFormat,
};
use tdlib::functions;
use tdlib::types::{
    Animation, Audio, ChatPhoto, Document, File, FormattedText, InlineKeyboardButton,
    InlineKeyboardButtonTypeUrl, Message, Photo, PhotoSize, ReplyMarkupInlineKeyboard, Sticker,
    UpdateChatMember, User, Video, VideoNote, VoiceNote,
};

use super::cache::CompactChat;
use crate::bot::TdResult;

pub trait MainUsername {
    fn main_username(&self) -> Option<&String>;
}

impl MainUsername for User {
    fn main_username(&self) -> Option<&String> {
        self.usernames.as_ref()?.active_usernames.first()
    }
}

pub enum MessageAttachment<'a> {
    Animation(Cow<'a, Animation>),
    Audio(Cow<'a, Audio>),
    Document(Cow<'a, Document>),
    Photo(Cow<'a, Photo>),
    Sticker(Cow<'a, Sticker>),
    Video(Cow<'a, Video>),
    VideoNote(Cow<'a, VideoNote>),
    VoiceNote(Cow<'a, VoiceNote>),
    ChatChangePhoto(Cow<'a, ChatPhoto>),
}

impl MessageAttachment<'_> {
    pub fn file(&self) -> &File {
        match self {
            Self::Animation(animation) => &animation.animation,
            Self::Audio(audio) => &audio.audio,
            Self::Document(document) => &document.document,
            Self::Photo(photo) => largest_photo(&photo.sizes).unwrap(),
            Self::Sticker(sticker) => &sticker.sticker,
            Self::Video(video) => &video.video,
            Self::VideoNote(video_note) => &video_note.video,
            Self::VoiceNote(voice_note) => &voice_note.voice,
            Self::ChatChangePhoto(chat_change_photo) => {
                largest_photo(&chat_change_photo.sizes).unwrap()
            }
        }
    }

    #[expect(clippy::missing_const_for_fn, reason = "incorrect suggestion")]
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Animation(animation) => &animation.mime_type,
            Self::Audio(audio) => &audio.mime_type,
            Self::Document(document) => &document.mime_type,
            Self::Photo(_) | Self::ChatChangePhoto(_) => "image/jpeg",
            Self::Sticker(sticker) => match sticker.format {
                StickerFormat::Webp => "image/webp",
                StickerFormat::Tgs => "application/x-tgsticker",
                StickerFormat::Webm => "video/webm",
            },
            Self::Video(video) => &video.mime_type,
            Self::VideoNote(_) => "video/mp4",
            Self::VoiceNote(voice_note) => &voice_note.mime_type,
        }
    }
}

pub const fn get_message_text(content: &MessageContent) -> Option<&FormattedText> {
    let formatted_text = match content {
        MessageContent::MessageText(message) => &message.text,
        MessageContent::MessageAnimation(message) => &message.caption,
        MessageContent::MessageAudio(message) => &message.caption,
        MessageContent::MessageDocument(message) => &message.caption,
        MessageContent::MessagePhoto(message) => &message.caption,
        MessageContent::MessageVideo(message) => &message.caption,
        MessageContent::MessageVoiceNote(message) => &message.caption,
        _ => return None,
    };

    Some(formatted_text)
}

pub fn get_message_attachment(
    content: Cow<'_, MessageContent>,
    include_non_images: bool,
) -> Option<MessageAttachment<'_>> {
    let attachment = match content {
        Cow::Borrowed(content) => match content {
            MessageContent::MessageAnimation(message) if include_non_images => {
                MessageAttachment::Animation(Cow::Borrowed(&message.animation))
            }
            MessageContent::MessageAudio(message) if include_non_images => {
                MessageAttachment::Audio(Cow::Borrowed(&message.audio))
            }
            MessageContent::MessageDocument(message) => {
                MessageAttachment::Document(Cow::Borrowed(&message.document))
            }
            MessageContent::MessagePhoto(message) => {
                MessageAttachment::Photo(Cow::Borrowed(&message.photo))
            }
            MessageContent::MessageSticker(message) => match message.sticker.format {
                StickerFormat::Webp => MessageAttachment::Sticker(Cow::Borrowed(&message.sticker)),
                StickerFormat::Tgs | StickerFormat::Webm if include_non_images => {
                    MessageAttachment::Sticker(Cow::Borrowed(&message.sticker))
                }
                _ => return None,
            },
            MessageContent::MessageVideo(message) if include_non_images => {
                MessageAttachment::Video(Cow::Borrowed(&message.video))
            }
            MessageContent::MessageVideoNote(message) if include_non_images => {
                MessageAttachment::VideoNote(Cow::Borrowed(&message.video_note))
            }
            MessageContent::MessageVoiceNote(message) if include_non_images => {
                MessageAttachment::VoiceNote(Cow::Borrowed(&message.voice_note))
            }
            MessageContent::MessageChatChangePhoto(message) => {
                MessageAttachment::ChatChangePhoto(Cow::Borrowed(&message.photo))
            }
            _ => return None,
        },
        Cow::Owned(content) => match content {
            MessageContent::MessageAnimation(message) => {
                MessageAttachment::Animation(Cow::Owned(message.animation))
            }
            MessageContent::MessageAudio(message) => {
                MessageAttachment::Audio(Cow::Owned(message.audio))
            }
            MessageContent::MessageDocument(message) => {
                MessageAttachment::Document(Cow::Owned(message.document))
            }
            MessageContent::MessagePhoto(message) => {
                MessageAttachment::Photo(Cow::Owned(message.photo))
            }
            MessageContent::MessageSticker(message) => match message.sticker.format {
                StickerFormat::Webp => MessageAttachment::Sticker(Cow::Owned(message.sticker)),
                StickerFormat::Tgs | StickerFormat::Webm if include_non_images => {
                    MessageAttachment::Sticker(Cow::Owned(message.sticker))
                }
                _ => return None,
            },
            MessageContent::MessageVideo(message) => {
                MessageAttachment::Video(Cow::Owned(message.video))
            }
            MessageContent::MessageVideoNote(message) => {
                MessageAttachment::VideoNote(Cow::Owned(message.video_note))
            }
            MessageContent::MessageVoiceNote(message) => {
                MessageAttachment::VoiceNote(Cow::Owned(message.voice_note))
            }
            MessageContent::MessageChatChangePhoto(message) => {
                MessageAttachment::ChatChangePhoto(Cow::Owned(message.photo))
            }
            _ => return None,
        },
    };

    Some(attachment)
}

fn largest_photo(sizes: &[PhotoSize]) -> Option<&File> {
    sizes
        .iter()
        .rfind(|photo_size| photo_size.photo.local.can_be_downloaded)
        .map(|photo_size| &photo_size.photo)
}

pub async fn get_message_or_reply_attachment(
    message: &Message,
    include_non_images: bool,
    client_id: i32,
) -> TdResult<Option<MessageAttachment>> {
    if let Some(attachment) =
        get_message_attachment(Cow::Borrowed(&message.content), include_non_images)
    {
        return Ok(Some(attachment));
    }

    let Some(MessageReplyTo::Message(reply)) = message.reply_to.as_ref() else {
        return Ok(None);
    };

    let content = if let Some(content) = reply.content.as_ref() {
        Cow::Borrowed(content)
    } else {
        let Some(enums::Message::Message(message)) =
            functions::get_replied_message(message.chat_id, message.id, client_id).await.ok()
        else {
            return Ok(None);
        };

        Cow::Owned(message.content)
    };

    Ok(get_message_attachment(content, include_non_images))
}

pub fn donate_markup(name: &str, url: impl Into<String>) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {name}"),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
}

pub fn log_status_update(update: &UpdateChatMember, chat: &CompactChat) {
    if let ChatType::Private(_) = chat.r#type {
        return;
    }

    if update.new_chat_member.status == update.old_chat_member.status {
        return;
    }

    let status = match update.new_chat_member.status {
        ChatMemberStatus::Member(_) => "joined",
        ChatMemberStatus::Left => "left",
        _ => return,
    };

    log::info!("{} {}", status, chat);
}
