#![feature(drain_filter)]
#![warn(clippy::pedantic)]

mod commands;
mod craiyon;
mod openai;
mod passwordpurgatory;
mod utils;

use std::env;
use std::error::Error;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;

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
            .chain(dptree::filter(|m: Message| m.forward().is_none()))
            .filter_command::<Command>()
            .endpoint(answer),
    )
    .default_handler(|_| async {})
    .dependencies(dptree::deps![reqwest::Client::new()])
    .worker_queue_size(16)
    .distribution_function::<()>(|_| None)
    .build()
    .setup_ctrlc_handler()
    .dispatch()
    .await;
}

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case")]
enum Command {
    #[command()]
    Start,
    #[command()]
    Generate(String),
    #[command()]
    Gpt3Code(String),
    #[command()]
    Password(String),
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
        Command::Generate(mut prompt) => {
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
            commands::generate(bot, message, prompt, http_client).await?;
        }
        Command::Gpt3Code(mut prompt) => {
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
            commands::gpt3_code(bot, message, prompt, http_client).await?;
        }
        Command::Password(password) => {
            if password.is_empty() {
                bot.send_message(message.chat.id, "Missing password.")
                    .reply_to_message_id(message.id)
                    .send()
                    .await
                    .ok();
                return Ok(());
            }
            commands::password(bot, message, password, http_client).await?;
        }
    };

    Ok(())
}
