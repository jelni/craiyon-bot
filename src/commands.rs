use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
pub use badtranslate::BadTranslate;
pub use charinfo::CharInfo;
pub use cobalt_download::CobaltDownload;
pub use generate::Generate;
pub use sex::Sex;
pub use start::Start;
pub use startit_joke::StartItJoke;
pub use translate::Translate;
pub use urbandictionary::UrbanDictionary;

use crate::utils::Context;

mod badtranslate;
mod charinfo;
mod cobalt_download;
mod generate;
mod sex;
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
