use serde::Serialize;
use tgbotapi::requests::{ChatID, ParseMode, ReplyMarkup};
use tgbotapi::{FileType, Message, TelegramRequest};

type RequestFiles = Option<Vec<(String, reqwest::multipart::Part)>>;

#[derive(Debug, Default, Serialize)]
pub struct SendPhoto {
    pub chat_id: ChatID,
    #[serde(skip_serializing_if = "FileType::needs_upload")]
    pub photo: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_sending_without_reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl TelegramRequest for SendPhoto {
    type Response = Message;

    fn endpoint(&self) -> &str {
        "sendPhoto"
    }

    fn files(&self) -> RequestFiles {
        if self.photo.needs_upload() {
            Some(vec![("photo".into(), self.photo.file().unwrap())])
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SendDocument {
    pub chat_id: ChatID,
    #[serde(skip_serializing_if = "FileType::needs_upload")]
    pub document: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_sending_without_reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl TelegramRequest for SendDocument {
    type Response = Message;

    fn endpoint(&self) -> &str {
        "sendDocument"
    }

    fn files(&self) -> RequestFiles {
        if self.document.needs_upload() {
            Some(vec![("document".into(), self.document.file().unwrap())])
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SendVoice {
    pub chat_id: ChatID,
    #[serde(skip_serializing_if = "FileType::needs_upload")]
    pub voice: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_sending_without_reply: Option<bool>,
}

impl TelegramRequest for SendVoice {
    type Response = Message;

    fn endpoint(&self) -> &str {
        "sendVoice"
    }

    fn files(&self) -> RequestFiles {
        if self.voice.needs_upload() {
            Some(vec![("voice".into(), self.voice.file().unwrap())])
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SendSticker {
    pub chat_id: ChatID,
    pub sticker: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i32>,
}

impl TelegramRequest for SendSticker {
    type Response = Message;

    fn endpoint(&self) -> &str {
        "sendSticker"
    }
}
