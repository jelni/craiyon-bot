use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::utils::{escape_markdown, Context, MARKDOWN_CHARS};

#[derive(Default)]
pub struct CharInfo;

#[async_trait]
impl CommandTrait for CharInfo {
    fn name(&self) -> &str {
        "charinfo"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chars = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("characters").await;
            return Ok(());
        };

        let mut lines = chars
            .chars()
            .take(11)
            .into_iter()
            .map(|c| {
                if c.is_ascii_whitespace() {
                    String::new()
                } else {
                    let cu32 = c as u32;
                    format!(
                        "`{}` `U\\+{:04X}` – `{}`",
                        if MARKDOWN_CHARS.contains(&c) { format!("\\{c}") } else { c.to_string() },
                        cu32,
                        escape_markdown(charname::get_name(cu32))
                    )
                }
            })
            .collect::<Vec<_>>();

        if lines.len() > 10 {
            lines.last_mut().unwrap().replace_range(.., "…");
        }

        ctx.reply_markdown(lines.join("\n")).await?;

        Ok(())
    }
}
