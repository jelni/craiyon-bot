use std::error::Error;

use async_trait::async_trait;

use super::Command;
use crate::poligon;
use crate::utils::Context;

pub struct StartItJoke;

#[async_trait]
impl Command for StartItJoke {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let joke = poligon::startit_joke(ctx.http_client.clone()).await?;
        ctx.reply(format!("Kacper Podpora mówi: {joke}")).await?;

        Ok(())
    }
}
