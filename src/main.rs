#![warn(clippy::pedantic)]

mod craiyon;
mod utils;

use std::env;
use std::error::Error;
use std::fmt::Write;
use std::io::Cursor;
use std::time::Instant;

use image::{ImageFormat, ImageOutputFormat};
use log::LevelFilter;
use reqwest::StatusCode;
use simple_logger::SimpleLogger;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageEntity};
use teloxide::utils::command::BotCommands;
use utils::CollageOptions;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let bot = Bot::new(env::var("TELEGRAM_TOKEN").unwrap());

    Dispatcher::builder(
        bot,
        dptree::entry().branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(answer),
        ),
    )
    .default_handler(|_| async {})
    .worker_queue_size(8)
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Start => {
            bot.send_message(message.chat.id, "siema").send().await?;
        }
        Command::Generate { prompt } => {
            generate(bot, message, prompt).await?;
        }
    };

    Ok(())
}

async fn generate(
    bot: Bot,
    message: Message,
    prompt: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let info_msg = bot
        .send_message(message.chat.id, "Generating…")
        .reply_to_message_id(message.id)
        .send()
        .await?;
    let start = Instant::now();
    match craiyon::generate(prompt.clone()).await {
        Ok(images) => {
            let duration = start.elapsed();

            let image = utils::image_collage(
                images.iter().map(|image| {
                    image::load_from_memory_with_format(image, ImageFormat::Jpeg).unwrap()
                }),
                CollageOptions {
                    image_count: (3, 3),
                    image_size: (256, 256),
                    gap: 8,
                },
            );

            let (mut caption, entities) = if prompt.is_empty() {
                ("Generated without a prompt".to_string(), Vec::new())
            } else {
                (
                    format!("Generated from prompt: {prompt}"),
                    Vec::from([MessageEntity::bold(23, prompt.chars().count())]),
                )
            };
            write!(caption, " in {}.", utils::format_duration(duration)).unwrap();

            let mut buffer = Cursor::new(Vec::new());
            image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

            bot.send_photo(message.chat.id, InputFile::memory(buffer.into_inner()))
                .caption(caption)
                .caption_entities(entities)
                .reply_to_message_id(message.id)
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
            .send()
            .await?;
        }
    };

    bot.delete_message(message.chat.id, info_msg.id)
        .send()
        .await?;

    Ok(())
}
