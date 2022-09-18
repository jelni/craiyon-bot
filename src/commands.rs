use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
pub use badtranslate::*;
pub use charinfo::*;
pub use cobalt_download::*;
pub use generate::*;
pub use ping::*;
pub use sex::*;
pub use stable_diffusion::*;
pub use start::*;
pub use startit_joke::*;
pub use translate::*;
pub use urbandictionary::*;

use crate::utils::Context;

mod badtranslate;
mod charinfo;
mod cobalt_download;
mod generate;
mod ping;
mod sex;
mod stable_diffusion;
mod start;
mod startit_joke;
mod translate;
mod urbandictionary;

#[async_trait]
pub trait Command {
    fn name(&self) -> &str;
    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
