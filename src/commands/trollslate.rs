use std::fmt::Write;
use std::iter;
use std::sync::Arc;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::google_translate;
use crate::utilities::google_translate::MissingTextToTranslate;
use crate::utilities::text_utils::EscapeMarkdown;

#[derive(Default)]
pub struct Trollslate;

#[async_trait]
impl CommandTrait for Trollslate {
    fn command_names(&self) -> &[&str] {
        &["trollslate", "troll"]
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let mut text = arguments.ok_or(MissingTextToTranslate)?;

        let language =
            if ctx.user.language_code.is_empty() { "en" } else { &ctx.user.language_code };

        let languages = [
            "am", "ar", "ca", "haw", "hi", "iw", "ja", "ka", "ko", "ru", "so", "sw", "xh", "zh-CN",
            "zu",
        ]
        .choose_multiple(&mut rand::thread_rng(), 9)
        .chain(iter::once(&language));

        let mut languages_str =
            format!("*{}*", google_translate::get_language_name(language).unwrap());

        for language in languages {
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
