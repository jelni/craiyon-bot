use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::text_utils::{EscapeChar, EscapeMarkdown};

pub struct CharInfo;

#[async_trait]
impl CommandTrait for CharInfo {
    fn command_names(&self) -> &[&str] {
        &["charinfo", "ch"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("get Unicode character names")
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(chars) = ConvertArgument::convert(ctx, &arguments).await?.0;
        let mut chars = chars.chars();

        let mut lines = chars
            .by_ref()
            .take(10)
            .map(|char| {
                if char.is_ascii_whitespace() {
                    String::new()
                } else {
                    let value = char.into();
                    format!(
                        "`{}` `U\\+{:04X}` – `{}`",
                        EscapeChar(char),
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
