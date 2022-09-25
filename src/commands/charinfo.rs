use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::utils::{escape_markdown, Context};

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
