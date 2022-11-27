use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use image::{imageops, DynamicImage};
use tdlib::enums::{
    self, ChatMemberStatus, InlineKeyboardButtonType, InputMessageContent, ReplyMarkup,
    TextEntityType, TextParseMode,
};
use tdlib::functions;
use tdlib::types::{
    FormattedText, InlineKeyboardButton, InlineKeyboardButtonTypeUrl, InputMessageText, Message,
    ReplyMarkupInlineKeyboard, TextParseModeMarkdown, UpdateChatMember, User,
};

use crate::bot::TdError;
use crate::commands::CommandTrait;
use crate::message_queue::MessageQueue;
use crate::ratelimit::RateLimiter;

pub const MARKDOWN_CHARS: [char; 20] = [
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!', '`',
    '\\',
];

pub type CommandRef = Box<dyn CommandTrait + Send + Sync>;

pub struct CommandInstance {
    pub name: &'static str,
    pub ratelimiter: Mutex<RateLimiter<i64>>,
    pub command_ref: CommandRef,
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub name: String,
    pub bot_username: Option<String>,
    pub arguments: Option<String>,
}

impl ParsedCommand {
    pub fn parse(formatted_text: &FormattedText) -> Option<ParsedCommand> {
        let entity = formatted_text
            .entities
            .iter()
            .find(|e| e.r#type == TextEntityType::BotCommand && e.offset == 0)?;

        let command = formatted_text
            .text
            .chars()
            .skip((entity.offset + 1).try_into().ok()?)
            .take((entity.length - 1).try_into().ok()?)
            .collect::<String>();
        let (command_name, username) = match command.split_once('@') {
            Some(parts) => (parts.0.into(), Some(parts.1)),
            None => (command, None),
        };
        let arguments = formatted_text
            .text
            .chars()
            .skip(entity.length.try_into().unwrap_or_default())
            .skip_while(char::is_ascii_whitespace)
            .collect::<String>();

        let arguments = if arguments.is_empty() { None } else { Some(arguments) };

        Some(ParsedCommand {
            name: command_name.to_lowercase(),
            bot_username: username.map(str::to_string),
            arguments,
        })
    }
}

pub struct RateLimits {
    pub ratelimit_exceeded: RateLimiter<i64>,
    pub auto_reply: RateLimiter<i64>,
}

pub struct Context {
    pub client_id: i32,
    pub message: Message,
    pub user: User,
    pub http_client: reqwest::Client,
    pub message_queue: Arc<MessageQueue>,
    pub ratelimits: Arc<Mutex<RateLimits>>,
}

impl Context {
    pub async fn reply_custom(
        &self,
        message_content: InputMessageContent,
        reply_markup: Option<enums::ReplyMarkup>,
    ) -> Result<Message, TdError> {
        let enums::Message::Message(message) = functions::send_message(
            self.message.chat_id,
            self.message.message_thread_id,
            self.message.id,
            None,
            reply_markup,
            message_content,
            self.client_id,
        )
        .await?;

        Ok(message)
    }

    async fn _reply_text(&self, text: FormattedText) -> Result<Message, TdError> {
        self.reply_custom(
            InputMessageContent::InputMessageText(InputMessageText {
                text,
                disable_web_page_preview: true,
                clear_draft: true,
            }),
            None,
        )
        .await
    }

    pub async fn reply<S: Into<String>>(&self, text: S) -> Result<Message, TdError> {
        self._reply_text(FormattedText { text: text.into(), ..Default::default() }).await
    }

    pub async fn reply_markdown<S: Into<String>>(&self, text: S) -> Result<Message, TdError> {
        let enums::FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            text.into(),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            self.client_id,
        )
        .await?;

        self._reply_text(formatted_text).await
    }

    pub async fn reply_html<S: Into<String>>(&self, text: S) -> Result<Message, TdError> {
        let enums::FormattedText::FormattedText(formatted_text) =
            functions::parse_text_entities(text.into(), TextParseMode::Html, self.client_id)
                .await?;

        self._reply_text(formatted_text).await
    }

    async fn _edit_message(
        &self,
        message_id: i64,
        text: FormattedText,
    ) -> Result<Message, TdError> {
        let enums::Message::Message(message) = functions::edit_message_text(
            self.message.chat_id,
            message_id,
            None,
            InputMessageContent::InputMessageText(InputMessageText {
                text,
                disable_web_page_preview: true,
                clear_draft: true,
            }),
            self.client_id,
        )
        .await?;

        Ok(message)
    }

    #[allow(dead_code)]
    pub async fn edit_message<S: Into<String>>(
        &self,
        message_id: i64,
        text: S,
    ) -> Result<Message, TdError> {
        self._edit_message(message_id, FormattedText { text: text.into(), ..Default::default() })
            .await
    }

    pub async fn edit_message_markdown<S: Into<String>>(
        &self,
        message_id: i64,
        text: S,
    ) -> Result<Message, TdError> {
        let enums::FormattedText::FormattedText(formatted_text) = functions::parse_text_entities(
            text.into(),
            TextParseMode::Markdown(TextParseModeMarkdown { version: 2 }),
            self.client_id,
        )
        .await?;

        self._edit_message(message_id, formatted_text).await
    }

    pub async fn delete_messages(&self, message_ids: Vec<i64>) -> Result<(), TdError> {
        functions::delete_messages(self.message.chat_id, message_ids, true, self.client_id).await
    }

    pub async fn delete_message(&self, message_id: i64) -> Result<(), TdError> {
        self.delete_messages(vec![message_id]).await
    }
}

pub trait DisplayUser {
    fn format_name(&self) -> String;
}

impl DisplayUser for User {
    fn format_name(&self) -> String {
        if self.username.is_empty() {
            if self.last_name.is_empty() {
                self.first_name.clone()
            } else {
                format!("{} {}", self.first_name, self.last_name)
            }
        } else {
            format!("@{}", self.username)
        }
    }
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

pub fn donate_markup<N: AsRef<str>, U: Into<String>>(name: N, url: U) -> ReplyMarkup {
    ReplyMarkup::InlineKeyboard(ReplyMarkupInlineKeyboard {
        rows: vec![vec![InlineKeyboardButton {
            text: format!("donate to {}", name.as_ref()),
            r#type: InlineKeyboardButtonType::Url(InlineKeyboardButtonTypeUrl { url: url.into() }),
        }]],
    })
}

pub async fn log_status_update(update: UpdateChatMember, client_id: i32) {
    let old_status = update.old_chat_member.status;
    let new_status = update.new_chat_member.status;

    if old_status == new_status {
        return;
    }

    let status = match new_status {
        ChatMemberStatus::Member => "joined",
        ChatMemberStatus::Left => "left",
        _ => return,
    };

    let enums::Chat::Chat(chat) = functions::get_chat(update.chat_id, client_id).await.unwrap();

    log::info!("{} {:?}", status, chat.title);
}
