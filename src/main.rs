#![warn(clippy::pedantic)]

use std::sync::Arc;

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

    bot.add_command(Arc::new(commands::start::Start::default()));
    bot.add_command(Arc::new(commands::ping::Ping::default()));
    bot.add_command(Arc::new(commands::generate::Generate::default()));
    bot.add_command(Arc::new(commands::stable_diffusion::StableDiffusion::default()));
    bot.add_command(Arc::new(commands::translate::Translate::default()));
    bot.add_command(Arc::new(commands::badtranslate::BadTranslate::default()));
    bot.add_command(Arc::new(commands::urbandictionary::UrbanDictionary::default()));
    bot.add_command(Arc::new(commands::cobalt_download::CobaltDownload::default()));
    bot.add_command(Arc::new(commands::charinfo::CharInfo::default()));
    bot.add_command(Arc::new(commands::startit_joke::StartItJoke::default()));
    bot.add_command(Arc::new(commands::kiwifarms::KiwiFarms::default()));
    bot.add_command(Arc::new(commands::sex::Sex::default()));

    bot.run().await;
}
