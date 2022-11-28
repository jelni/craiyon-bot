use std::time::Duration;

use rand::Rng;
use tdlib::enums::{InputMessageContent, MessageContent};
use tdlib::functions;
use tdlib::types::{FormattedText, InputMessageText, Message, MessageDice};

pub async fn accurate(message: Message, client_id: i32) {
    if let MessageContent::MessageDice(dice) = message.content {
        let text = if dice_success(&dice) {
            "accurate"
        } else if rand::thread_rng().gen_bool(1. / 20.) {
            "skill issue"
        } else {
            return;
        };

        tokio::time::sleep(Duration::from_secs(3)).await;

        functions::send_message(
            message.chat_id,
            message.message_thread_id,
            message.id,
            None,
            None,
            InputMessageContent::InputMessageText(InputMessageText {
                text: FormattedText { text: text.into(), ..Default::default() },
                disable_web_page_preview: true,
                clear_draft: true,
            }),
            client_id,
        )
        .await
        .unwrap();
    }
}

fn dice_success(dice: &MessageDice) -> bool {
    match (dice.emoji.as_str(), dice.value) {
        ("🎲" | "🎯" | "🎳", 6) | ("🏀", 4..) | ("⚽", 3..) => true,
        ("🎰", _) => dice.value % 21 == 1,
        _ => false,
    }
}
