use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::utils::{escape_markdown, Context};

pub struct CharInfo;

#[async_trait]
impl Command for CharInfo {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chars = match &ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("characters").await;
                return Ok(());
            }
        };

        let mut lines = chars
            .chars()
            .into_iter()
            .map(|c| {
                if c.is_ascii_whitespace() {
                    String::new()
                } else {
                    format!("`{}` `U\\+{:04X}`", escape_markdown(c.to_string()), c as u32)
                }
            })
            .collect::<Vec<_>>();

        if lines.len() > 10 {
            lines.truncate(10);
            lines.push(String::from('…'));
        }

        ctx.reply_markdown(lines.join("\n")).await?;

        Ok(())
    }
}
