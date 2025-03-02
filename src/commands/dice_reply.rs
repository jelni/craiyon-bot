use std::time::Duration;

use rand::Rng;
use tdlib::enums::{InputMessageContent, InputMessageReplyTo, MessageContent};
use tdlib::functions;
use tdlib::types::{
    FormattedText, InputMessageReplyToMessage, InputMessageText, LinkPreviewOptions, Message,
    MessageDice,
};

pub async fn execute(message: Message, client_id: i32) {
    let MessageContent::MessageDice(dice) = message.content else {
        return;
    };

    let text = if dice_success(&dice) {
        "accurate"
    } else if rand::rng().random_bool(1. / 10.) {
        "skill issue"
    } else {
        return;
    };

    tokio::time::sleep(Duration::from_secs(3)).await;

    functions::send_message(
        message.chat_id,
        message.message_thread_id,
        Some(InputMessageReplyTo::Message(InputMessageReplyToMessage {
            message_id: message.id,
            ..Default::default()
        })),
        None,
        None,
        InputMessageContent::InputMessageText(InputMessageText {
            text: FormattedText { text: text.into(), ..Default::default() },
            link_preview_options: Some(LinkPreviewOptions {
                is_disabled: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        client_id,
    )
    .await
    .unwrap();
}

fn dice_success(dice: &MessageDice) -> bool {
    match (dice.emoji.as_str(), dice.value) {
        ("🎲" | "🎯" | "🎳", 6) | ("🏀", 4..) | ("⚽", 3..) => true,
        ("🎰", _) => dice.value % 21 == 1,
        _ => false,
    }
}
