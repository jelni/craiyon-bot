use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;

use super::Command;
use crate::apis::kiwifarms;
use crate::utils::Context;

pub struct KiwiFarms;

#[async_trait]
impl Command for KiwiFarms {
    fn name(&self) -> &str {
        "does_kiwifarms_work"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = match kiwifarms::status(ctx.http_client.clone()).await {
            Ok(status) => {
                if status == StatusCode::OK {
                    "yes 🤬".to_string()
                } else {
                    format!("{} no", status.as_u16())
                }
            }
            Err(err) => err.without_url().to_string(),
        };
        ctx.reply(text).await?;

        Ok(())
    }
}