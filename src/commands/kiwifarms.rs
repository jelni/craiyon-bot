use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;

use super::CommandTrait;
use crate::apis::kiwifarms;
use crate::utils::Context;

#[derive(Default)]
pub struct KiwiFarms;

#[async_trait]
impl CommandTrait for KiwiFarms {
    fn name(&self) -> &str {
        "does_kiwifarms_work"
    }

    fn aliases(&self) -> &[&str] {
        &["kf"]
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = match kiwifarms::status(ctx.http_client.clone()).await {
            Ok(status) => {
                if status == StatusCode::OK || status == StatusCode::FOUND {
                    "yes 🤬".to_string()
                } else {
                    format!("{} no", status.as_u16())
                }
            }
            Err(err) => format!("no ({})", err.without_url()),
        };
        ctx.reply(text).await?;

        Ok(())
    }
}
