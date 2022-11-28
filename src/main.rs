#![warn(clippy::pedantic)]

use bot::Bot;

mod apis;
mod bot;
mod command_context;
mod command_manager;
mod commands;
mod logchamp;
mod message_queue;
mod not_commands;
mod parsed_command;
mod ratelimit;
mod utils;

#[tokio::main]
async fn main() {
    logchamp::init();
    dotenv::dotenv().ok();

    let mut bot = Bot::new();

    bot.add_command(Box::<commands::start::Start>::default());
    bot.add_command(Box::<commands::generate::Generate>::default());
    bot.add_command(Box::new(commands::stablehorde::StableHorde::stable_diffusion_2()));
    bot.add_command(Box::new(commands::stablehorde::StableHorde::stable_diffusion()));
    bot.add_command(Box::new(commands::stablehorde::StableHorde::waifu_diffusion()));
    bot.add_command(Box::new(commands::stablehorde::StableHorde::furry_diffusion()));
    bot.add_command(Box::<commands::translate::Translate>::default());
    bot.add_command(Box::<commands::badtranslate::BadTranslate>::default());
    bot.add_command(Box::<commands::urbandictionary::UrbanDictionary>::default());
    bot.add_command(Box::<commands::screenshot::Screenshot>::default());
    bot.add_command(Box::<commands::cobalt_download::CobaltDownload>::default());
    bot.add_command(Box::<commands::charinfo::CharInfo>::default());
    bot.add_command(Box::<commands::autocomplete::Autocomplete>::default());
    bot.add_command(Box::<commands::tts::Tts>::default());
    bot.add_command(Box::<commands::kiwifarms::KiwiFarms>::default());
    bot.add_command(Box::<commands::startit_joke::StartItJoke>::default());
    bot.add_command(Box::<commands::kebab::Kebab>::default());
    bot.add_command(Box::<commands::ping::Ping>::default());
    bot.add_command(Box::<commands::delete::Delete>::default());
    bot.add_command(Box::<commands::sex::Sex>::default());

    bot.run().await;
}
