#![warn(clippy::pedantic)]

use std::sync::Arc;

use bot::Bot;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod api_methods;
mod bot;
mod cobalt;
mod commands;
mod craiyon;
mod mathjs;
mod not_commands;
mod poligon;
mod ratelimit;
mod translate;
mod urbandictionary;
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

    bot.add_command(Arc::new(commands::Start));
    bot.add_command(Arc::new(commands::Generate));
    bot.add_command(Arc::new(commands::Translate));
    bot.add_command(Arc::new(commands::BadTranslate));
    bot.add_command(Arc::new(commands::UrbanDictionary));
    bot.add_command(Arc::new(commands::CobaltDownload));
    bot.add_command(Arc::new(commands::CharInfo));
    bot.add_command(Arc::new(commands::StartItJoke));
    bot.add_command(Arc::new(commands::Sex));

    bot.run().await;
}
