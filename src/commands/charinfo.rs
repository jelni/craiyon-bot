use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::command_context::CommandContext;
use crate::utils::{escape_markdown, MARKDOWN_CHARS};

#[derive(Default)]
pub struct CharInfo;

#[async_trait]
impl CommandTrait for CharInfo {
    fn command_names(&self) -> &[&str] {
        &["charinfo", "ch"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get Unicode character names")
    }

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult {
        let chars = arguments.ok_or(MissingArgument("characters"))?;
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
                        if MARKDOWN_CHARS.contains(&char) {
                            format!("\\{char}")
                        } else {
                            char.into()
                        },
                        value,
                        escape_markdown(charname::get_name(value))
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
