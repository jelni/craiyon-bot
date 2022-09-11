use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::Command;
use crate::poligon;
use crate::utils::Context;

pub struct StartItJoke;

#[async_trait]
impl Command for StartItJoke {
    fn name(&self) -> &str {
        "startit_joke"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        _: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let joke = poligon::startit_joke(ctx.http_client.clone()).await?;
        ctx.reply(format!("Kacper Podpora mówi: {joke}")).await?;

        Ok(())
    }
}
