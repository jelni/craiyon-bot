use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{StatusCode, Url};
use tgbotapi::FileType;

use super::CommandError::{CustomMarkdownError, MissingArgument};
use super::{CommandResult, CommandTrait};
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

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let media_url = arguments.ok_or(MissingArgument("URL to download"))?;

        let mut urls = cobalt::query(ctx.http_client.clone(), media_url.clone()).await??;

        let status_msg = ctx.reply("downloading…").await?;

        urls.truncate(4);
        let mut downloads = Vec::with_capacity(urls.len());

        for url in urls {
            match cobalt::download(ctx.http_client.clone(), url).await {
                Ok(download) if download.media.is_empty() => {
                    Err("≫ cobalt failed to download media. try again later.")?;
                }
                Ok(download) => downloads.push(download),
                Err(err) => {
                    Err(err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string())?;
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
                let text = "could not upload media to Telegram\\. you can [download it here]";
                let url =
                    Url::parse_with_params("https://co.wukko.me/", [("u", &media_url)]).unwrap();
                Err(CustomMarkdownError(format!("{text}({url})\\.")))?;
            }
        }

        ctx.delete_message(&status_msg).await.ok();

        Ok(())
    }
}
