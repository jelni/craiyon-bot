#![warn(clippy::pedantic)]

use bot::Bot;

mod api_methods;
mod apis;
mod bot;
mod commands;
mod logchamp;
mod not_commands;
mod ratelimit;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    logchamp::init();
    dotenv::dotenv().ok();

    let mut bot = Bot::new().await;

    bot.add_command(Box::<commands::start::Start>::default());
    bot.add_command(Box::<commands::ping::Ping>::default());
    bot.add_command(Box::<commands::generate::Generate>::default());
    bot.add_command(Box::new(commands::stable_horde::StableHorde::stable_diffusion()));
    bot.add_command(Box::new(commands::stable_horde::StableHorde::waifu_diffusion()));
    bot.add_command(Box::new(commands::stable_horde::StableHorde::furry_diffusion()));
    bot.add_command(Box::<commands::translate::Translate>::default());
    bot.add_command(Box::<commands::badtranslate::BadTranslate>::default());
    bot.add_command(Box::<commands::tts::Tts>::default());
    bot.add_command(Box::<commands::urbandictionary::UrbanDictionary>::default());
    bot.add_command(Box::<commands::screenshot::Screenshot>::default());
    bot.add_command(Box::<commands::cobalt_download::CobaltDownload>::default());
    bot.add_command(Box::<commands::charinfo::CharInfo>::default());
    bot.add_command(Box::<commands::delete::Delete>::default());
    bot.add_command(Box::<commands::startit_joke::StartItJoke>::default());
    bot.add_command(Box::<commands::autocomplete::Autocomplete>::default());
    bot.add_command(Box::<commands::kiwifarms::KiwiFarms>::default());
    bot.add_command(Box::<commands::sex::Sex>::default());

    bot.run().await;
}
