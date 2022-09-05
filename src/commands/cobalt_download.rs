use std::error::Error;

use async_trait::async_trait;
use reqwest::{StatusCode, Url};
use tgbotapi::requests::{DeleteMessage, ParseMode, SendDocument, SendMessage};
use tgbotapi::FileType;

use super::Command;
use crate::cobalt;
use crate::utils::{donate_markup, Context};

pub struct CobaltDownload;

#[async_trait]
impl Command for CobaltDownload {
    async fn execute(&self, ctx: Context) -> Result<(), Box<dyn Error>> {
        let media_url = match ctx.arguments {
            Some(arguments) => arguments,
            None => {
                ctx.missing_argument("URL to download").await;
                return Ok(());
            }
        };

        match cobalt::query(ctx.http_client.clone(), &media_url).await? {
            Ok(url) => {
                let status_msg = ctx
                    .api
                    .make_request(&SendMessage {
                        chat_id: ctx.message.chat_id(),
                        text: "Downloading…".to_string(),
                        reply_to_message_id: Some(ctx.message.message_id),
                        ..Default::default()
                    })
                    .await?
                    .message_id;

                match cobalt::download(ctx.http_client, url).await {
                    Ok(download) if download.media.is_empty() => {
                        ctx.api
                            .make_request(&SendMessage {
                                chat_id: ctx.message.chat_id(),
                                text: "≫ cobalt failed to download media. Try again later."
                                    .to_string(),
                                ..Default::default()
                            })
                            .await?;
                    }
                    Ok(download) => {
                        if ctx
                            .api
                            .make_request(&SendDocument {
                                chat_id: ctx.message.chat_id(),
                                document: FileType::Bytes(download.filename, download.media),
                                reply_to_message_id: Some(ctx.message.message_id),
                                // missing `allow_sending_without_reply`!
                                reply_markup: Some(donate_markup(
                                    "≫ cobalt",
                                    "https://boosty.to/wukko",
                                )),
                                ..Default::default()
                            })
                            .await
                            .is_err()
                        {
                            let text =
                                "Could not upload media to Telegram\\. You can [download it here]";
                            let url =
                                Url::parse_with_params("https://co.wukko.me/", [("u", media_url)])
                                    .unwrap();
                            ctx.api
                                .make_request(&SendMessage {
                                    chat_id: ctx.message.chat_id(),
                                    text: format!("{text}({url})\\."),
                                    parse_mode: Some(ParseMode::MarkdownV2),
                                    reply_to_message_id: Some(ctx.message.message_id),
                                    ..Default::default()
                                })
                                .await?;
                        }
                    }
                    Err(err) => {
                        ctx.api
                            .make_request(&SendMessage {
                                chat_id: ctx.message.chat_id(),
                                text: err
                                    .status()
                                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                                    .to_string(),
                                reply_to_message_id: Some(ctx.message.message_id),
                                ..Default::default()
                            })
                            .await?;
                    }
                }

                ctx.api
                    .make_request(&DeleteMessage {
                        chat_id: ctx.message.chat_id(),
                        message_id: status_msg,
                    })
                    .await?;
            }
            Err(text) => {
                ctx.api
                    .make_request(&SendMessage {
                        chat_id: ctx.message.chat_id(),
                        text,
                        reply_to_message_id: Some(ctx.message.message_id),
                        ..Default::default()
                    })
                    .await?;
            }
        }

        Ok(())
    }
}
