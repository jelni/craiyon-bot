use image::{imageops, DynamicImage};
use tdlib::enums::{
    self, ChatMemberStatus, ChatType, InlineKeyboardButtonType, MessageContent, ReplyMarkup,
};
use tdlib::functions;
use tdlib::types::{
    File, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, Message, ReplyMarkupInlineKeyboard,
    UpdateChatMember,
};

use crate::cache::CompactChat;
use crate::ratelimit::RateLimiter;

pub const MARKDOWN_CHARS: [char; 20] = [
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!', '`',
    '\\',
];

pub struct RateLimits {
    pub ratelimit_exceeded: RateLimiter<i64>,
}

pub trait TruncateWithEllipsis {
    fn truncate_with_ellipsis(&mut self, max_len: usize);
}

impl TruncateWithEllipsis for String {
    fn truncate_with_ellipsis(&mut self, max_len: usize) {
        if self.chars().count() > max_len {
            self.truncate(max_len - 1);
            self.push('…');
        }
    }
}

pub fn check_prompt<S: AsRef<str>>(prompt: S) -> Option<&'static str> {
    let prompt = prompt.as_ref();
    if prompt.chars().count() > 512 {
        Some("this prompt is too long.")
    } else if prompt.lines().count() > 4 {
        Some("this prompt has too many lines.")
    } else {
        None
    }
}

pub fn image_collage(
    images: Vec<DynamicImage>,
    image_size: (u32, u32),
    image_count_x: u32,
    gap: u32,
) -> DynamicImage {
    #[allow(clippy::pedantic)] // multiple lossy numeric conversions
    let image_count_y = (images.len() as f32 / image_count_x as f32).ceil() as u32;
    let mut base = DynamicImage::new_rgb8(
        image_count_x * image_size.0 + (image_count_x - 1) * gap,
        image_count_y * image_size.1 + (image_count_y - 1) * gap,
    );

    for (i, image) in images.into_iter().enumerate() {
        let col = i % image_count_x as usize;
        let row = i / image_count_x as usize;
        let x = col * (image_size.0 + gap) as usize;
        let y = row * (image_size.1 + gap) as usize;
        imageops::overlay(&mut base, &image, x as _, y as _);
    }

    base
}

pub fn format_duration(duration: u64) -> String {
    let hours = (duration / 3600) % 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub fn escape_markdown<S: AsRef<str>>(text: S) -> String {
    let text = text.as_ref();
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if MARKDOWN_CHARS.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

pub fn get_message_image(message: &Message) -> Option<File> {
    match &message.content {
        MessageContent::MessageDocument(document) => Some(document.document.document.clone()),
        MessageContent::MessagePhoto(photo) => {
            photo.photo.sizes.last().map(|photo_size| photo_size.photo.clone())
        }
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

pub fn donate_markup<N: AsRef<str>, U: Into<String>>(name: N, url: U) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {}", name.as_ref()),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
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
