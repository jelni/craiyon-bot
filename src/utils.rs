use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use image::{imageops, DynamicImage};
use tgbotapi::requests::{DeleteMessage, ParseMode, ReplyMarkup, SendMessage};
use tgbotapi::{
    InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageEntityType, Telegram, User,
};

const MARKDOWN_CHARS: [char; 18] =
    ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];

#[allow(clippy::unreadable_literal)]
const RABBIT_JE: i64 = -1001722954366;

#[derive(Debug)]
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

#[derive(Clone)]
pub struct Context {
    pub api: Arc<Telegram>,
    pub message: Message,
    pub arguments: Option<String>,
    pub http_client: reqwest::Client,
}

impl Context {
    pub async fn missing_argument<S: AsRef<str>>(&self, argument: S) {
        self.reply(format!("Missing {}.", argument.as_ref())).await.ok();
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

#[derive(Clone, Copy)]
pub struct CollageOptions {
    pub image_count: (u32, u32),
    pub image_size: (u32, u32),
    pub gap: u32,
}

pub fn image_collage<I: IntoIterator<Item = DynamicImage>>(
    images: I,
    options: CollageOptions,
) -> DynamicImage {
    let size = (
        options.image_count.0 * options.image_size.0 + (options.image_count.0 - 1) * options.gap,
        options.image_count.1 * options.image_size.1 + (options.image_count.1 - 1) * options.gap,
    );
    let mut base = DynamicImage::new_rgb8(size.0, size.1);

    for (i, image) in images.into_iter().enumerate() {
        let col = i % options.image_count.0 as usize;
        let row = i / options.image_count.0 as usize;
        let x = col * (options.image_size.0 + options.gap) as usize;
        let y = row * (options.image_size.1 + options.gap) as usize;
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

pub async fn rabbit_nie_je(ctx: Context) -> Result<(), Result<(), Box<dyn Error>>> {
    if let Some(chat) = &ctx.message.forward_from_chat {
        if chat.id == RABBIT_JE {
            let result = match ctx
                .api
                .make_request(&DeleteMessage {
                    chat_id: ctx.message.chat_id(),
                    message_id: ctx.message.message_id,
                })
                .await
            {
                Ok(_) => "Deleted",
                Err(_) => "Couldn't delete",
            };
            log::warn!(
                "{result} a message from in {:?}",
                chat.title.as_deref().unwrap_or_default()
            );
        }
    }

    Ok(())
}
