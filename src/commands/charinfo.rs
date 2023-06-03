use async_trait::async_trait;

use super::{CommandResult, CommandTrait};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::message_entities::{self, ToEntity, ToEntityOwned};

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

        let mut entities = chars
            .by_ref()
            .take(10)
            .flat_map(|char| {
                if char.is_ascii_whitespace() {
                    vec!["\n".text()]
                } else {
                    let value = char.into();
                    vec![
                        "\n".text(),
                        char.to_string().code_owned(),
                        " ".text(),
                        format!("U+{value:04X}").code_owned(),
                        " – ".text(),
                        charname::get_name(value).code(),
                    ]
                }
            })
            .skip(1)
            .collect::<Vec<_>>();

        if chars.next().is_some() {
            entities.push("…".text());
        }

        ctx.reply_formatted_text(message_entities::formatted_text(entities)).await?;

        Ok(())
    }
}
