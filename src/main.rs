#![warn(clippy::pedantic)]
#![feature(iter_intersperse)]

use bot::Bot;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod api_methods;
mod apis;
mod bot;
mod commands;
mod not_commands;
mod ratelimit;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("craiyon_bot", LevelFilter::Debug)
        .init()
        .unwrap();

    let mut bot = Bot::new().await;

    bot.add_command(Box::new(commands::start::Start::default()));
    bot.add_command(Box::new(commands::ping::Ping::default()));
    bot.add_command(Box::new(commands::generate::Generate::default()));
    bot.add_command(Box::new(commands::stable_diffusion::StableDiffusion::default()));
    bot.add_command(Box::new(commands::translate::Translate::default()));
    bot.add_command(Box::new(commands::badtranslate::BadTranslate::default()));
    bot.add_command(Box::new(commands::tts::Tts::default()));
    bot.add_command(Box::new(commands::urbandictionary::UrbanDictionary::default()));
    bot.add_command(Box::new(commands::cobalt_download::CobaltDownload::default()));
    bot.add_command(Box::new(commands::charinfo::CharInfo::default()));
    bot.add_command(Box::new(commands::delete::Delete::default()));
    bot.add_command(Box::new(commands::startit_joke::StartItJoke::default()));
    bot.add_command(Box::new(commands::kiwifarms::KiwiFarms::default()));
    bot.add_command(Box::new(commands::sex::Sex::default()));

    bot.run().await;
}
