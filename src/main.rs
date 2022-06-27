#![warn(clippy::pedantic)]

mod craiyon;
mod utils;
use std::env;
use std::error::Error;
use std::io::Cursor;

use image::{ImageFormat, ImageOutputFormat};
use log::LevelFilter;
use reqwest::StatusCode;
use simple_logger::SimpleLogger;
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode, User};
use teloxide::utils::command::BotCommands;
use teloxide::utils::markdown;
use utils::CollageOptions;

const HELP_TEXT: &str = "Use the /generate command to generate images\\.
*Example:* `/generate crayons in a box`";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let bot = Bot::new(env::var("TELEGRAM_TOKEN").unwrap());

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(answer),
    )
    .default_handler(|_| async {})
    .dependencies(dptree::deps![reqwest::Client::new()])
    .worker_queue_size(16)
    .distribution_function(|update| update.user().map(|u| u.id))
    .build()
    .setup_ctrlc_handler()
    .dispatch()
    .await;
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase")]
enum Command {
    #[command()]
    Start,
    #[command()]
    Generate { prompt: String },
}

async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
    http_client: reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Start => {
            bot.send_message(message.chat.id, HELP_TEXT)
                .parse_mode(ParseMode::MarkdownV2)
                .send()
                .await?;
        }
        Command::Generate { mut prompt } => {
            if prompt.is_empty() {
                if let Some(text) = message.reply_to_message().and_then(Message::text) {
                    prompt = text.to_string();
                } else {
                    bot.send_message(message.chat.id, "Missing prompt.")
                        .reply_to_message_id(message.id)
                        .send()
                        .await
                        .ok();
                    return Ok(());
                }
            }
            generate(bot, message, prompt, http_client).await?;
        }
    };

    Ok(())
}

async fn generate(
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
            let image = utils::image_collage(
                result.images.iter().map(|image| {
                    image::load_from_memory_with_format(image, ImageFormat::Jpeg).unwrap()
                }),
                CollageOptions {
                    image_count: (3, 3),
                    image_size: (256, 256),
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
