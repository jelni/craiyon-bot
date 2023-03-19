use std::sync::Arc;

use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::StringGreedyOrReply;
use crate::utilities::parse_arguments::ParseArguments;
use crate::utilities::text_utils::{self, EscapeMarkdown};

pub struct CharInfo;

#[async_trait]
impl CommandTrait for CharInfo {
    fn command_names(&self) -> &[&str] {
        &["charinfo", "ch"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get Unicode character names")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: String) -> CommandResult {
        let StringGreedyOrReply(chars) =
            ParseArguments::parse_arguments(ctx.clone(), &arguments).await?;
        let mut chars = chars.chars();

        let mut lines = chars
            .by_ref()
            .take(10)
            .map(|char| {
                if char.is_ascii_whitespace() {
                    String::new()
                } else {
                    let value = char as u32;
                    format!(
                        "`{}` `U\\+{:04X}` – `{}`",
                        if text_utils::MARKDOWN_CHARS.contains(&char) {
                            format!("\\{char}")
                        } else {
                            char.into()
                        },
                        value,
                        EscapeMarkdown(charname::get_name(value))
                    )
                }
            })
            .collect::<Vec<_>>();

        if chars.next().is_some() {
            lines.push("…".into());
        }

        ctx.reply_markdown(lines.join("\n")).await?;

        Ok(())
    }
}
