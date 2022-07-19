#![feature(drain_filter)]
#![warn(clippy::pedantic)]

mod commands;
mod craiyon;
mod openai;
mod urbandictionary;
mod utils;

use std::env;
use std::error::Error;

use log::LevelFilter;
use reqwest::redirect;
use simple_logger::SimpleLogger;
use teloxide::dptree;
use teloxide::prelude::*;
use teloxide::types::{ForwardedFrom, ParseMode};
use teloxide::utils::command::BotCommands;

#[allow(clippy::unreadable_literal)]
const RABBIT_JE: i64 = -1001722954366;
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

    let http_client = reqwest::Client::builder()
        .redirect(redirect::Policy::none())
        .build()
        .unwrap();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .branch(
                dptree::filter(|m: Message| m.forward().is_none())
                    .filter_command::<Command>()
                    .endpoint(answer),
            )
            .branch(dptree::endpoint(rabbit_nie_je)),
    )
    .default_handler(|_| async {})
    .dependencies(dptree::deps![http_client])
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
    UrbanDictionary(String),
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
        Command::UrbanDictionary(term) => {
            if term.is_empty() {
                bot.send_message(message.chat.id, "Missing word to define.")
                    .reply_to_message_id(message.id)
                    .send()
                    .await
                    .ok();
                return Ok(());
            }
            commands::urbandictionary(bot, message, term, http_client).await?;
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
    };

    Ok(())
}

async fn rabbit_nie_je(bot: Bot, message: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(forward) = message.forward() {
        if let ForwardedFrom::Chat(chat) = &forward.from {
            {
                if chat.id.0 == RABBIT_JE {
                    let result = match bot.delete_message(message.chat.id, message.id).send().await
                    {
                        Ok(_) => "Deleted",
                        Err(_) => "Couldn't delete",
                    };
                    log::warn!(
                        "{result} a message from {:?} in {:?}",
                        chat.title().unwrap_or_default(),
                        message.chat.title().unwrap_or_default()
                    );
                }
            }
        }
    }

    Ok(())
}
