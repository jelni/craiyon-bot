#![warn(clippy::pedantic)]

mod craiyon;

use std::env;
use std::error::Error;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, MessageEntity};
use teloxide::utils::command::BotCommands;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let bot = Bot::new(env::var("TELEGRAM_TOKEN").unwrap());
    teloxide::commands_repl(bot, answer, Command::ty()).await;
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start the bot")]
    Start,
    #[command(description = "generate images")]
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
    let images = craiyon::generate(prompt.clone()).await?;
    bot.send_media_group(
        message.chat.id,
        images.into_iter().map(|image| {
            InputMedia::Photo(
                InputMediaPhoto::new(InputFile::memory(image))
                    .caption(format!("Generated from: {prompt}"))
                    .caption_entities([MessageEntity::bold(17, prompt.len())]),
            )
        }),
    )
    .reply_to_message_id(message.id)
    .send()
    .await?;
    bot.delete_message(message.chat.id, info_msg.id)
        .send()
        .await?;

    Ok(())
}
