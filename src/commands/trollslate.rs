use std::iter;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use super::{CommandResult, CommandTrait};
use crate::apis::translate;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::google_translate::Language;
use crate::utilities::message_entities::ToEntity;
use crate::utilities::{google_translate, message_entities};

pub struct Trollslate;

#[async_trait]
impl CommandTrait for Trollslate {
    fn command_names(&self) -> &[&str] {
        &["trollslate", "troll"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("translate text through many random languages")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let (target_language, StringGreedyOrReply(text)) =
            <(Option<Language>, _)>::convert(ctx, &arguments).await?.0;

        let mut languages = [
            "am", "ar", "ca", "cy", "haw", "hi", "iw", "ja", "ka", "ko", "ru", "si", "so", "sw",
            "xh", "zh-CN", "zu",
        ]
        .choose_multiple(&mut rand::thread_rng(), 9);

        let next_language = languages.next().unwrap();
        let translation =
            translate::single(ctx.http_client.clone(), &text, None, next_language).await?;
        let mut text = translation.text;
        let source_language = translation.source_language;

        let mut entities = vec![
            google_translate::get_language_name(&source_language)
                .unwrap_or(&source_language)
                .bold(),
            " ➜ ".text(),
            google_translate::get_language_name(next_language).unwrap_or(next_language).bold(),
        ];

        for language in languages.copied().chain(iter::once(
            target_language.map_or(source_language.as_str(), |target_language| target_language.0),
        )) {
            text = translate::single(ctx.http_client.clone(), &text, None, language).await?.text;
            entities.extend([
                " ➜ ".text(),
                google_translate::get_language_name(language).unwrap_or(language).bold(),
            ]);
        }

        entities.extend(["\n".text(), text.text()]);

        ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;

        Ok(())
    }
}
