use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use crate::ratelimit::RateLimiter;
use crate::utils::Context;

pub mod badtranslate;
pub mod charinfo;
pub mod cobalt_download;
pub mod generate;
pub mod kiwifarms;
pub mod ping;
pub mod sex;
pub mod stable_diffusion;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod urbandictionary;

#[async_trait]
pub trait CommandTrait {
    fn name(&self) -> &str;
    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(4, 20)
    }
    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
