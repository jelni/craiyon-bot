use std::error::Error;

use async_trait::async_trait;
use reqwest::StatusCode;

use super::{CommandResult, CommandTrait};
use crate::apis::kiwifarms;
use crate::utilities::command_context::CommandContext;

pub struct KiwiFarms;

#[async_trait]
impl CommandTrait for KiwiFarms {
    fn command_names(&self) -> &[&str] {
        &["does_kiwifarms_work", "kiwifarms", "kf"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("checks if The Kiwi Farms forum still works")
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.send_typing().await?;

        let text = match kiwifarms::status(ctx.bot_state.http_client.clone()).await {
            Ok(status) => {
                if status == StatusCode::NON_AUTHORITATIVE_INFORMATION {
                    "yes 🤬".into()
                } else {
                    format!("{} no", status.as_u16())
                }
            }
            Err(err) => {
                let err = err.without_url();
                format!("no ({})", err.source().map_or(&err as &dyn Error, |err| err))
            }
        };
        ctx.reply(text).await?;

        Ok(())
    }
}
