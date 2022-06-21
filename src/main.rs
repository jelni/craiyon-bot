#![warn(clippy::pedantic)]

mod craiyon;

use std::env;
use std::error::Error;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, MessageEntity};
use teloxide::utils::command::BotCommands;

const ERROR_TEXT: &str = "zjebalo sie";

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
    .distribution_function::<()>(|_| None)
    .build()
    .setup_ctrlc_handler()
    .dispatch()
    .await;
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
    match craiyon::generate(prompt.clone()).await {
        Ok(images) => {
            bot.send_media_group(
                message.chat.id,
                images.into_iter().map(|image| {
                    InputMedia::Photo(
                        InputMediaPhoto::new(InputFile::memory(image))
                            .caption(format!("Generated from prompt: {prompt}"))
                            .caption_entities([MessageEntity::bold(23, prompt.len())]),
                    )
                }),
            )
            .reply_to_message_id(message.id)
            .send()
            .await?;
        }
        Err(_) => {
            bot.send_message(message.chat.id, ERROR_TEXT).send().await?;
        }
    };

    bot.delete_message(message.chat.id, info_msg.id)
        .send()
        .await?;

    Ok(())
}
