use bot::Bot;
use utilities::logchamp;

mod apis;
mod bot;
mod commands;
mod utilities;

#[tokio::main]
async fn main() {
    logchamp::init();
    dotenvy::dotenv().ok();

    let mut bot = Bot::new();

    bot.add_command(commands::start::Start);
    bot.add_command(commands::craiyon::Generate);
    bot.add_command(commands::craiyon::Craiyon::art());
    bot.add_command(commands::craiyon::Craiyon::drawing());
    bot.add_command(commands::craiyon::Craiyon::photo());
    bot.add_command(commands::craiyon::Craiyon::none());
    bot.add_command(commands::craiyon_search::CraiyonSearch);
    bot.add_command(commands::fal::Fal::fast_sdxl());
    bot.add_command(commands::fal::Fal::sana());
    bot.add_command(commands::stablehorde::StableHorde::stable_diffusion());
    bot.add_command(commands::stablehorde::StableHorde::stable_diffusion_2());
    bot.add_command(commands::stablehorde::StableHorde::waifu_diffusion());
    bot.add_command(commands::stablehorde::StableHorde::furry_diffusion());
    bot.add_command(commands::markov_chain::MarkovChain);
    bot.add_command(commands::config::Config);
    bot.add_command(commands::different_dimension_me::DifferentDimensionMe);
    bot.add_command(commands::gemini::Gemini::gemini());
    bot.add_command(commands::gemini::Gemini::gemini2());
    bot.add_command(commands::groq::Groq::llama());
    bot.add_command(commands::groq::Groq::thinker());
    bot.add_command(commands::translate::Translate);
    bot.add_command(commands::badtranslate::BadTranslate);
    bot.add_command(commands::trollslate::Trollslate);
    bot.add_command(commands::urbandictionary::UrbanDictionary);
    bot.add_command(commands::screenshot::Screenshot);
    bot.add_command(commands::cobalt_download::CobaltDownload::auto());
    bot.add_command(commands::cobalt_download::CobaltDownload::audio());
    bot.add_command(commands::yt_dlp::YtDlp::video());
    bot.add_command(commands::yt_dlp::YtDlp::audio());
    bot.add_command(commands::charinfo::CharInfo);
    bot.add_command(commands::radio_sur::RadioSur);
    bot.add_command(commands::radio_poligon::RadioPoligon);
    bot.add_command(commands::autocomplete::Autocomplete);
    bot.add_command(commands::petittube::Petittube);
    bot.add_command(commands::mevo::Mevo);
    bot.add_command(commands::kiwifarms::KiwiFarms);
    bot.add_command(commands::startit_joke::StartitJoke);
    bot.add_command(commands::moveit_joke::MoveitJoke);
    bot.add_command(commands::kebab::Kebab);
    bot.add_command(commands::ping::Ping);
    bot.add_command(commands::delete::Delete);
    bot.add_command(commands::sex::Sex);

    bot.run();
    log::logger().flush();
}
