use tdlib::enums::{InputInlineQueryResult, InputMessageContent};
use tdlib::functions;
use tdlib::types::{
    FormattedText, InputInlineQueryResultArticle, InputMessageText, UpdateNewInlineQuery,
};

use crate::apis::mathjs;

pub async fn execute(query: UpdateNewInlineQuery, http_client: reqwest::Client, client_id: i32) {
    let (query_id, query) = (query.id, query.query);

    if query.is_empty() {
        functions::answer_inline_query(
            query_id,
            false,
            None,
            Vec::new(),
            3600,
            String::new(),
            client_id,
        )
        .await
        .ok();

        return;
    }

    let (title, message_text) = if query.split_ascii_whitespace().collect::<String>() == "2+2" {
        ("5".into(), format!("{query} = 5"))
    } else {
        match mathjs::evaluate(http_client.clone(), query.clone()).await.unwrap() {
            Ok(result) => (result.clone(), format!("{query} = {result}")),
            Err(err) => (err.clone(), err),
        }
    };

    functions::answer_inline_query(
        query_id,
        false,
        None,
        vec![InputInlineQueryResult::Article(InputInlineQueryResultArticle {
            id: "0".into(),
            url: String::new(),
            hide_url: true,
            title,
            description: String::new(),
            thumbnail_url: String::new(),
            thumbnail_width: 0,
            thumbnail_height: 0,
            reply_markup: None,
            input_message_content: InputMessageContent::InputMessageText(InputMessageText {
                text: FormattedText { text: message_text, ..Default::default() },
                ..Default::default()
            }),
        })],
        3600,
        String::new(),
        client_id,
    )
    .await
    .ok();
}
