use std::fmt::Write;
use std::iter;
use std::sync::Arc;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedyOrReply;
use crate::utilities::google_translate;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::text_utils::EscapeMarkdown;

pub struct Trollslate;

#[async_trait]
impl CommandTrait for Trollslate {
    fn command_names(&self) -> &[&str] {
        &["trollslate", "troll"]
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: String) -> CommandResult {
        let StringGreedyOrReply(text) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;

        let mut languages = [
            "am", "ar", "ca", "haw", "hi", "iw", "ja", "ka", "ko", "ru", "so", "sw", "xh", "zh-CN",
            "zu",
        ]
        .choose_multiple(&mut rand::thread_rng(), 9);

        let next_language = languages.next().unwrap();
        let translation =
            translate::single(ctx.http_client.clone(), text, None, next_language).await?;
        let mut text = translation.text;
        let source_language = translation.source_language;

        let mut languages_str = format!(
            "*{}* ➜ *{}*",
            EscapeMarkdown(google_translate::get_language_name(&source_language).unwrap()),
            EscapeMarkdown(google_translate::get_language_name(next_language).unwrap())
        );

        for language in languages.chain(iter::once(&source_language.as_str())) {
            text = translate::single(ctx.http_client.clone(), text, None, language).await?.text;
            write!(
                languages_str,
                " ➜ *{}*",
                EscapeMarkdown(google_translate::get_language_name(language).unwrap())
            )
            .unwrap();
        }

        ctx.reply_markdown(format!("{languages_str}\n{}", EscapeMarkdown(&text))).await?;

        Ok(())
    }
}
