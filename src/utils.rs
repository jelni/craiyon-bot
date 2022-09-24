use std::convert::TryInto;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use image::{imageops, DynamicImage};
use tgbotapi::requests::{
    DeleteMessage, EditMessageText, MessageOrBool, ParseMode, ReplyMarkup, SendMessage,
};
use tgbotapi::{
    ChatMemberStatus, ChatMemberUpdated, ChatType, FileType, InlineKeyboardButton,
    InlineKeyboardMarkup, Message, MessageEntityType, Telegram, User,
};

use crate::api_methods::SendSticker;
use crate::commands::Command;
use crate::ratelimit::RateLimiter;

const MARKDOWN_CHARS: [char; 18] =
    ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];

// yes, people generated all of these
const DISALLOWED_WORDS: [&str; 37] = [
    "abuse", "anus", "ass", "bikini", "boob", "booba", "boobs", "braless", "breast", "breasts",
    "butt", "butts", "cum", "dick", "doujin", "erotic", "hentai", "incest", "lingerie", "loli",
    "lolicon", "lolis", "naked", "nhentai", "nude", "penis", "porn", "porno", "rape", "sex",
    "sexy", "shota", "shotacon", "slut", "tits", "underage", "xxx",
];

pub type CommandRef = Box<dyn Command + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub name: String,
    pub bot_username: Option<String>,
    pub arguments: Option<String>,
}

impl ParsedCommand {
    pub fn parse(message: &Message) -> Option<ParsedCommand> {
        message.entities.clone().and_then(|entities| {
            entities
                .into_iter()
                .find(|e| e.entity_type == MessageEntityType::BotCommand && e.offset == 0)
                .map(|e| {
                    let command = message
                        .text
                        .clone()
                        .unwrap()
                        .chars()
                        .skip((e.offset + 1).try_into().unwrap_or_default())
                        .take((e.length - 1).try_into().unwrap_or_default())
                        .collect::<String>();
                    let (command_name, username) = match command.split_once('@') {
                        Some(parts) => (parts.0.to_string(), Some(parts.1)),
                        None => (command, None),
                    };
                    let arguments = message
                        .text
                        .clone()
                        .unwrap()
                        .chars()
                        .skip(e.length.try_into().unwrap_or_default())
                        .collect::<String>()
                        .trim_start()
                        .to_string();

                    let arguments = if arguments.is_empty() { None } else { Some(arguments) };

                    ParsedCommand {
                        name: command_name,
                        bot_username: username.map(str::to_string),
                        arguments,
                    }
                })
        })
    }

    pub fn normalised_name(&self) -> String {
        self.name.to_lowercase()
    }
}

impl fmt::Display for ParsedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.normalised_name())?;
        if let Some(arguments) = &self.arguments {
            write!(f, " {arguments:?}")?;
        }

        Ok(())
    }
}

pub struct Context {
    pub api: Arc<Telegram>,
    pub message: Message,
    pub user: User,
    pub http_client: reqwest::Client,
    pub global_ratelimiter: Arc<RwLock<RateLimiter<(i64, &'static str)>>>,
}

impl Context {
    pub async fn missing_argument<S: AsRef<str>>(&self, argument: S) {
        self.reply(format!("Missing {}.", argument.as_ref())).await.ok();
    }

    async fn _reply<S: Into<String>>(
        &self,
        text: S,
        parse_mode: Option<ParseMode>,
    ) -> Result<Message, tgbotapi::Error> {
        self.api
            .make_request(&SendMessage {
                chat_id: self.message.chat_id(),
                text: text.into(),
                parse_mode,
                disable_web_page_preview: Some(true),
                reply_to_message_id: Some(self.message.message_id),
                ..Default::default()
            })
            .await
    }

    pub async fn reply<S: Into<String>>(&self, text: S) -> Result<Message, tgbotapi::Error> {
        self._reply(text, None).await
    }

    pub async fn reply_markdown<S: Into<String>>(
        &self,
        text: S,
    ) -> Result<Message, tgbotapi::Error> {
        self._reply(text, Some(ParseMode::MarkdownV2)).await
    }

    async fn _edit_message<S: Into<String>>(
        &self,
        message: &Message,
        text: S,
        parse_mode: Option<ParseMode>,
    ) -> Result<MessageOrBool, tgbotapi::Error> {
        self.api
            .make_request(&EditMessageText {
                chat_id: message.chat_id(),
                message_id: Some(message.message_id),
                text: text.into(),
                parse_mode,
                disable_web_page_preview: Some(true),
                ..Default::default()
            })
            .await
    }

    pub async fn edit_message<S: Into<String>>(
        &self,
        message: &Message,
        text: S,
    ) -> Result<MessageOrBool, tgbotapi::Error> {
        self._edit_message(message, text, None).await
    }

    pub async fn edit_message_markdown<S: Into<String>>(
        &self,
        message: &Message,
        text: S,
    ) -> Result<MessageOrBool, tgbotapi::Error> {
        self._edit_message(message, text, Some(ParseMode::MarkdownV2)).await
    }

    pub async fn delete_message(&self, message: &Message) -> Result<bool, tgbotapi::Error> {
        self.api
            .make_request(&DeleteMessage {
                chat_id: message.chat_id(),
                message_id: message.message_id,
            })
            .await
    }

    pub async fn send_sticker(&self, sticker: FileType) -> Result<Message, tgbotapi::Error> {
        self.api
            .make_request(&SendSticker {
                chat_id: self.message.chat_id(),
                sticker,
                reply_to_message_id: Some(self.message.message_id),
            })
            .await
    }
}

pub trait DisplayUser {
    fn format_name(&self) -> String;
}

impl DisplayUser for User {
    fn format_name(&self) -> String {
        match &self.username {
            Some(username) => format!("@{username}"),
            None => match &self.last_name {
                Some(last_name) => format!("{} {last_name}", self.first_name),
                None => self.first_name.clone(),
            },
        }
    }
}

pub fn check_prompt<S: AsRef<str>>(prompt: S) -> Option<&'static str> {
    let prompt = prompt.as_ref();
    if prompt.chars().count() > 1024 {
        Some("This prompt is too long.")
    } else if prompt.lines().count() > 5 {
        Some("This prompt has too many lines.")
    } else if is_prompt_suspicious(prompt) {
        Some("This prompt is sus.")
    } else {
        None
    }
}

fn is_prompt_suspicious<S: AsRef<str>>(text: S) -> bool {
    text.as_ref()
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .any(|w| DISALLOWED_WORDS.contains(&w))
}

pub fn image_collage(images: Vec<DynamicImage>, image_count_x: u32, gap: u32) -> DynamicImage {
    let (image_size_x, image_size_y) = {
        let image = images.first().unwrap();
        (image.width(), image.height())
    };
    #[allow(clippy::pedantic)] // multiple lossy numeric conversions
    let image_count_y = (images.len() as f32 / image_count_x as f32).ceil() as u32;
    let mut base = DynamicImage::new_rgb8(
        image_count_x * image_size_x + (image_count_x - 1) * gap,
        image_count_y * image_size_y + (image_count_y - 1) * gap,
    );

    for (i, image) in images.into_iter().enumerate() {
        let col = i % image_count_x as usize;
        let row = i / image_count_x as usize;
        let x = col * (image_size_x + gap) as usize;
        let y = row * (image_size_y + gap) as usize;
        imageops::overlay(&mut base, &image, x as _, y as _);
    }

    base
}

pub fn format_duration(duration: Duration) -> String {
    let duration = duration.as_secs();
    let hours = (duration / 3600) % 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
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

pub fn donate_markup<N: AsRef<str>, U: Into<String>>(name: N, url: U) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton {
            text: format!("Donate to {}", name.as_ref()),
            url: Some(url.into()),
            ..Default::default()
        }]],
    })
}

pub fn log_status_update(update: ChatMemberUpdated) {
    if update.chat.chat_type == ChatType::Private {
        return;
    }

    let old_status = update.old_chat_member.status;
    let new_status = update.new_chat_member.status;

    if old_status == new_status {
        return;
    }

    let status = match new_status {
        ChatMemberStatus::Member => "Joined",
        ChatMemberStatus::Left | ChatMemberStatus::Kicked => "Left",
        _ => return,
    };

    log::info!("{} {:?}", status, update.chat.title.unwrap_or_default());
}
