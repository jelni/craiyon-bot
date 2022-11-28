use tdlib::enums::{InputMessageContent, MessageContent};
use tdlib::functions;
use tdlib::types::{FormattedText, InputMessageText, Message, MessageDice};

pub async fn accurate(message: Message, client_id: i32) {
    if let MessageContent::MessageDice(dice) = message.content {
        if !filter_dice(&dice) {
            return;
        }

        functions::send_message(
            message.chat_id,
            message.message_thread_id,
            message.id,
            None,
            None,
            InputMessageContent::InputMessageText(InputMessageText {
                text: FormattedText { text: "accurate".into(), ..Default::default() },
                disable_web_page_preview: true,
                clear_draft: true,
            }),
            client_id,
        )
        .await
        .unwrap();
    }
}

fn filter_dice(dice: &MessageDice) -> bool {
    match (dice.emoji.as_str(), dice.value) {
        ("🎲" | "🎯" | "🎳", 6) | ("🏀", 4..) | ("⚽", 3..) => true,
        ("🎰", _) => dice.value % 21 == 1,
        _ => false,
    }
}
