use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{StatusCode, Url};
use tgbotapi::FileType;

use super::CommandTrait;
use crate::api_methods::SendDocument;
use crate::apis::cobalt;
use crate::utils::{donate_markup, Context};

#[derive(Default)]
pub struct CobaltDownload;

#[async_trait]
impl CommandTrait for CobaltDownload {
    fn name(&self) -> &'static str {
        "cobalt_download"
    }

    fn aliases(&self) -> &[&str] {
        &["cobalt", "download", "dl"]
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(media_url) = arguments else {
            ctx.missing_argument("URL to download").await;
            return Ok(());
        };

        let mut urls = match cobalt::query(ctx.http_client.clone(), &media_url).await? {
            Ok(urls) => urls,
            Err(text) => {
                ctx.reply(text).await?;
                return Ok(());
            }
        };

        let status_msg = ctx.reply("Downloading…").await?;

        urls.truncate(4);
        let mut downloads = Vec::with_capacity(urls.len());

        for url in urls {
            match cobalt::download(ctx.http_client.clone(), url).await {
                Ok(download) if download.media.is_empty() => {
                    ctx.reply("≫ cobalt failed to download media. Try again later.").await?;
                    return Ok(());
                }
                Ok(download) => downloads.push(download),
                Err(err) => {
                    ctx.reply(
                        err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string(),
                    )
                    .await?;
                    return Ok(());
                }
            }
        }

        for download in downloads {
            if ctx
                .api
                .make_request(&SendDocument {
                    chat_id: ctx.message.chat_id(),
                    document: FileType::Bytes(download.filename, download.media),
                    reply_to_message_id: Some(ctx.message.message_id),
                    allow_sending_without_reply: Some(true),
                    reply_markup: Some(donate_markup("≫ cobalt", "https://boosty.to/wukko")),
                    ..Default::default()
                })
                .await
                .is_err()
            {
                let text = "Could not upload media to Telegram\\. You can [download it here]";
                let url =
                    Url::parse_with_params("https://co.wukko.me/", [("u", media_url)]).unwrap();
                ctx.reply_markdown(format!("{text}({url})\\.")).await?;
                return Ok(());
            }
        }

        ctx.delete_message(&status_msg).await.ok();

        Ok(())
    }
}
