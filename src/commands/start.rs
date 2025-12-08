use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::message_entities::{self, ToEntity};

pub struct Start;

#[async_trait]
impl CommandTrait for Start {
    fn command_names(&self) -> &[&str] {
        &["start", "help"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply_formatted_text(message_entities::formatted_text(vec![
            concat!(
                "hello! this bot contains many useful and fun commands like:\n",
                "• /gemini (/g) – talk to Google Gemini\n",
                "• /cobalt_download (/dl), /cobalt_download_audio (/dla) – download media using ",
            )
            .text(),
            "≫ cobalt".text_url("https://cobalt.tools/"),
            "\n• /yt_dlp (/yt), /yt_dlp_audio (/yta) – download media using ".text(),
            "yt-dlp".text_url("https://github.com/yt-dlp/yt-dlp"),
            concat!(
                "\n• /translate (/tr) – translate text using Google Translate\n",
                "• /convert (/c) – convert between popular currencies and cryptocurrencies\n",
                "• /screenshot (/ss) – screenshot websites\n",
                "• /urbandictionary (/ud) – get slang term definitions\n",
                "• /charinfo (/ch) – see Unicode character names\n",
                "and more!\n\n",
                "you can reply to other messages ",
                "or quote their parts to provide arguments. ",
                "this is my open-source hobby project, made using ",
            )
            .text(),
            "Rust".text_url("https://www.rust-lang.org/"),
            " and ".text(),
            "TDLib".text_url("https://core.telegram.org/tdlib"),
            ". not everything will always work. star the repository on ".text(),
            "GitHub".text_url("https://github.com/jelni/craiyon-bot"),
            "!\n- @zuzia".text(),
        ]))
        .await?;

        Ok(())
    }
}
