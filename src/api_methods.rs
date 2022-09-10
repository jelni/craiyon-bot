use serde::Serialize;
use tgbotapi::requests::ChatID;
use tgbotapi::{FileType, Message, TelegramRequest};

#[derive(Serialize, Default, Debug)]
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
