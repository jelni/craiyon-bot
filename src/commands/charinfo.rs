use std::sync::Arc;

use async_trait::async_trait;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::utils::{escape_markdown, Context, MARKDOWN_CHARS};

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

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let chars = arguments.ok_or(MissingArgument("characters"))?;

        let mut lines = chars
            .chars()
            .take(10)
            .into_iter()
            .map(|c| {
                if c.is_ascii_whitespace() {
                    String::new()
                } else {
                    let cu32 = c as u32;
                    format!(
                        "`{}` `U\\+{:04X}` – `{}`",
                        if MARKDOWN_CHARS.contains(&c) { format!("\\{c}") } else { c.into() },
                        cu32,
                        escape_markdown(charname::get_name(cu32))
                    )
                }
            })
            .collect::<Vec<_>>();

        if chars.chars().count() > 10 {
            lines.push("…".into());
        }

        ctx.reply_markdown(lines.join("\n")).await?;

        Ok(())
    }
}
