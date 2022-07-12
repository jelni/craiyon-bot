use std::error::Error;
use std::io::Cursor;

use image::{ImageFormat, ImageOutputFormat};
use reqwest::StatusCode;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageEntity, ParseMode, User};
use teloxide::utils::markdown;

use crate::utils::CollageOptions;
use crate::{craiyon, openai, passwordpurgatory, utils};

pub async fn generate(
    bot: Bot,
    message: Message,
    prompt: String,
    http_client: reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if prompt.chars().count() > 1024 {
        bot.send_message(message.chat.id, "This prompt is too long.")
            .reply_to_message_id(message.id)
            .send()
            .await
            .ok();
        return Ok(());
    };

    let info_msg = match bot
        .send_message(message.chat.id, format!("Generating {prompt}…"))
        .reply_to_message_id(message.id)
        .send()
        .await
    {
        Ok(message) => message,
        Err(_) => return Ok(()), // this usually means that the original message was deleted
    };

    log::info!(
        "Generating {prompt:?} for {user_name} ({user_id}) in {chat_name} ({chat_id})",
        user_name = message
            .from()
            .map_or_else(|| "-".to_string(), User::full_name),
        user_id = message.from().map(|u| u.id.0).unwrap_or_default(),
        chat_name = message.chat.title().unwrap_or("-"),
        chat_id = message.chat.id
    );

    match craiyon::generate(http_client, prompt.clone()).await {
        Ok(result) => {
            let images = result
                .images
                .iter()
                .flat_map(|image| image::load_from_memory_with_format(image, ImageFormat::Jpeg))
                .collect::<Vec<_>>();
            let image_size = {
                let image = images.first().unwrap();
                (image.width(), image.height())
            };
            let image = utils::image_collage(
                images,
                CollageOptions {
                    image_count: (3, 3),
                    image_size,
                    gap: 8,
                },
            );

            let mut buffer = Cursor::new(Vec::new());
            image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

            bot.send_photo(message.chat.id, InputFile::memory(buffer.into_inner()))
                .caption(format!(
                    "Generated from prompt: *{}* in {}\\.",
                    markdown::escape(&prompt),
                    utils::format_duration(result.duration)
                ))
                .parse_mode(ParseMode::MarkdownV2)
                .reply_to_message_id(message.id)
                .allow_sending_without_reply(true)
                .send()
                .await?;
        }
        Err(err) => {
            bot.send_message(
                message.chat.id,
                format!(
                    "zjebalo sie: {}",
                    err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                ),
            )
            .reply_to_message_id(message.id)
            .allow_sending_without_reply(true)
            .send()
            .await?;
        }
    };

    bot.delete_message(message.chat.id, info_msg.id)
        .send()
        .await
        .ok();

    Ok(())
}

pub async fn gpt3_code(
    bot: Bot,
    message: Message,
    prompt: String,
    http_client: reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (text, entities) = match openai::complete_code(
        http_client,
        openai::Config {
            prompt,
            max_tokens: 256,
            temperature: Some(1.),
            stop: None,
        },
    )
    .await
    {
        Ok(text) => {
            let len = text.chars().count();
            (text, Vec::from([MessageEntity::pre(None, 0, len)]))
        }
        Err(_) => ("zjebalo sie".to_string(), Vec::new()),
    };
    bot.send_message(message.chat.id, text)
        .entities(entities)
        .reply_to_message_id(message.id)
        .allow_sending_without_reply(true)
        .send()
        .await?;

    Ok(())
}

pub async fn password(
    bot: Bot,
    message: Message,
    password: String,
    http_client: reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = passwordpurgatory::make_hell(http_client, password).await?;
    bot.send_message(message.chat.id, text)
        .reply_to_message_id(message.id)
        .send()
        .await
        .ok();

    Ok(())
}
