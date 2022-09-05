use std::error::Error;

use async_trait::async_trait;

use crate::utils::Context;

pub mod badtranslate;
pub mod charinfo;
pub mod cobalt_download;
pub mod generate;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod urbandictionary;

#[async_trait]
pub trait Command {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>>;
}
