use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use super::CommandTrait;
use crate::apis::poligon;
use crate::utils::Context;

#[derive(Default)]
pub struct StartItJoke;

#[async_trait]
impl CommandTrait for StartItJoke {
    fn name(&self) -> &str {
        "startit_joke"
    }

    fn aliases(&self) -> &[&str] {
        &["startit"]
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
