use std::time::Duration;

use rand::Rng;
use tdlib::enums::{InputMessageContent, MessageContent, MessageReplyTo};
use tdlib::functions;
use tdlib::types::{FormattedText, InputMessageText, Message, MessageDice, MessageReplyToMessage};

pub async fn execute(message: Message, client_id: i32) {
    let MessageContent::MessageDice(dice) = message.content else {
        return;
    };

    let text = if dice_success(&dice) {
        "accurate"
    } else if rand::thread_rng().gen_bool(1. / 10.) {
        "skill issue"
    } else {
        return;
    };

    tokio::time::sleep(Duration::from_secs(3)).await;

    functions::send_message(
        message.chat_id,
        message.message_thread_id,
        Some(MessageReplyTo::Message(MessageReplyToMessage {
            chat_id: message.chat_id,
            message_id: message.id,
        })),
        None,
        None,
        InputMessageContent::InputMessageText(InputMessageText {
            text: FormattedText { text: text.into(), ..Default::default() },
            disable_web_page_preview: true,
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
