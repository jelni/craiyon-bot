use std::sync::Arc;

use tgbotapi::requests::{
    AnswerInlineQuery, InlineQueryResult, InlineQueryResultArticle, InlineQueryType,
    InputMessageText, InputMessageType,
};
use tgbotapi::{InlineQuery, Telegram};

use crate::mathjs;

pub async fn calculate_inline(
    api: Arc<Telegram>,
    http_client: reqwest::Client,
    inline_query: InlineQuery,
) {
    let query = inline_query.query;
    if query.is_empty() {
        api.make_request(&AnswerInlineQuery {
            inline_query_id: inline_query.id,
            results: Vec::new(),
            ..Default::default()
        })
        .await
        .ok();

        return;
    }

    let (title, message_text) = if query.split_ascii_whitespace().collect::<String>() == "2+2" {
        ("5".to_string(), format!("{} = 5", query))
    } else {
        match mathjs::evaluate(http_client.clone(), query.clone()).await {
            Ok(result) => match result {
                Ok(result) => (result.clone(), format!("{} = {}", query, result)),
                Err(err) => (err.clone(), err),
            },
            Err(err) => {
                log::error!("{err}");
                return;
            }
        }
    };

    api.make_request(&AnswerInlineQuery {
        inline_query_id: inline_query.id,
        results: vec![InlineQueryResult {
            id: "0".to_string(),
            result_type: "article".to_string(),
            content: InlineQueryType::Article(InlineQueryResultArticle {
                title,
                input_message_content: InputMessageType::Text(InputMessageText {
                    message_text,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reply_markup: None,
        }],
        ..Default::default()
    })
    .await
    .ok();
}
