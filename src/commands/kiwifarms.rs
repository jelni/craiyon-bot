use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;

use super::{CommandResult, CommandTrait};
use crate::apis::kiwifarms;
use crate::utils::Context;

#[derive(Default)]
pub struct KiwiFarms;

#[async_trait]
impl CommandTrait for KiwiFarms {
    fn name(&self) -> &'static str {
        "does_kiwifarms_work"
    }

    fn aliases(&self) -> &[&str] {
        &["kf"]
    }

    async fn execute(&self, ctx: Arc<Context>, _: Option<String>) -> CommandResult {
        let text = match kiwifarms::status(ctx.http_client.clone()).await {
            Ok(status) => {
                if status == StatusCode::OK || status == StatusCode::FOUND {
                    "yes 🤬".into()
                } else {
                    format!("{} no", status.as_u16())
                }
            }
            Err(err) => {
                let err = err.without_url();
                format!(
                    "no ({})",
                    match err.source() {
                        Some(err) => err,
                        None => &err as _,
                    }
                )
            }
        };
        ctx.reply(text).await?;

        Ok(())
    }
}
