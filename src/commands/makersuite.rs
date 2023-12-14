use std::fmt::Write;
use std::fs;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tdlib::enums::{File, MessageContent};
use tdlib::types::FormattedText;
use tdlib::{enums, functions};

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::makersuite::{self, Blob, CitationSource, Part, PartResponse};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, StringGreedyOrReply};
use crate::utilities::file_download::MEBIBYTE;
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::telegram_utils;

pub struct GoogleGemini;

#[async_trait]
impl CommandTrait for GoogleGemini {
    fn command_names(&self) -> &[&str] {
        &["gemini"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask Gemini Pro or Gemini Pro Vision")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 45)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let prompt = Option::<StringGreedyOrReply>::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let mut model = "gemini-pro";
        let mut parts = Vec::new();

        if let Some(prompt) = prompt {
            parts.push(Part::Text(prompt.0));
        }

        if let Some(mut file) =
            telegram_utils::get_message_or_reply_image(&ctx.message, ctx.client_id).await
        {
            if file.expected_size > 4 * MEBIBYTE {
                return Err(CommandError::Custom("the image cannot be larger than 4 MiB.".into()));
            }

            let mime_type = if let MessageContent::MessageDocument(document) = &ctx.message.content
            {
                if !["image/png", "image/jpeg", "image/heic", "image/heif", "image/webp"]
                    .contains(&document.document.mime_type.as_str())
                {
                    return Err(CommandError::Custom(
                        "only PNG, JPEG, HEIC, HEIF, and WebP files are supported.".into(),
                    ));
                }

                &document.document.mime_type
            } else {
                "image/jpeg"
            };

            model = "gemini-pro-vision";

            File::File(file) =
                functions::download_file(file.id, 1, 0, 0, true, ctx.client_id).await?;

            let file = fs::read(file.local.path).unwrap();

            parts.push(Part::InlineData(Blob { mime_type, data: STANDARD.encode(file) }));
        }

        if parts.is_empty() {
            return Err(CommandError::Custom("no prompt or image provided.".into()));
        }

        let response =
            makersuite::generate_content(ctx.bot_state.http_client.clone(), model, &parts, 512)
                .await?;

        let response = match response {
            Ok(response) => response,
            Err(response) => {
                return Err(CommandError::Custom(format!(
                    "error {}: {}",
                    response.error.code, response.error.message
                )));
            }
        };

        if let Some(prompt_feedback) = response.prompt_feedback {
            if let Some(block_reason) = prompt_feedback.block_reason {
                if block_reason == "SAFETY" {
                    let reasons = prompt_feedback
                        .safety_ratings
                        .into_iter()
                        .filter(|safety_rating| safety_rating.blocked.unwrap_or_default())
                        .map(|safety_rating| safety_rating.category)
                        .collect::<Vec<_>>()
                        .join(", ");

                    return Err(CommandError::Custom(format!(
                        "request blocked by Google: {reasons}."
                    )));
                };

                return Err(CommandError::Custom("request blocked by Google.".into()));
            };
        }

        let Some(candidate) = response.candidates.into_iter().next() else {
            return Err(CommandError::Custom("no response generated.".into()));
        };

        let mut text = candidate
            .content
            .parts
            .into_iter()
            .map(|part| match part {
                PartResponse::Text(text) => text,
                PartResponse::InlineData(_) => "[unsupported response part]".into(),
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        if candidate.finish_reason != "STOP" {
            write!(text, " [{}]", candidate.finish_reason).unwrap();
        }

        if let Some(citation_metadata) = candidate.citation_metadata {
            writeln!(text).unwrap();
            write!(text, "\n{}", format_citations(citation_metadata.citation_sources)).unwrap();
        }

        let enums::FormattedText::FormattedText(formatted_text) =
            functions::parse_markdown(FormattedText { text, ..Default::default() }, ctx.client_id)
                .await?;

        ctx.reply_formatted_text(formatted_text).await?;

        Ok(())
    }
}

pub struct GooglePalm;

#[async_trait]
impl CommandTrait for GooglePalm {
    fn command_names(&self) -> &[&str] {
        &["palm", "palm2"]
    }

    fn description(&self) -> Option<&'static str> {
        Some("ask Google PaLM 2 (Legacy)")
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 45)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let StringGreedyOrReply(prompt) = ConvertArgument::convert(ctx, &arguments).await?.0;

        ctx.send_typing().await?;

        let response =
            makersuite::generate_text(ctx.bot_state.http_client.clone(), &prompt, 512).await?;

        let response = match response {
            Ok(response) => response,
            Err(response) => {
                return Err(CommandError::Custom(format!(
                    "error {}: {}",
                    response.error.code, response.error.message
                )));
            }
        };

        if let Some(filters) = response.filters {
            let reasons = filters
                .into_iter()
                .map(|filter| {
                    if let Some(message) = filter.message {
                        format!("{}: {message}", filter.reason)
                    } else {
                        filter.reason
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            ctx.reply(format!("request blocked by Google: {reasons}.",)).await?;
            return Ok(());
        }

        let Some(candidate) =
            response.candidates.and_then(|candidates| candidates.into_iter().next())
        else {
            return Err(CommandError::Custom("no response generated.".into()));
        };

        if candidate.output.is_empty() {
            return Err(CommandError::Custom("no text generated.".into()));
        }

        let mut text = candidate.output;

        if let Some(citation_metadata) = candidate.citation_metadata {
            writeln!(text).unwrap();
            write!(text, "\n{}", format_citations(citation_metadata.citation_sources)).unwrap();
        }

        let enums::FormattedText::FormattedText(formatted_text) =
            functions::parse_markdown(FormattedText { text, ..Default::default() }, ctx.client_id)
                .await?;

        ctx.reply_formatted_text(formatted_text).await?;

        Ok(())
    }
}

fn format_citations(citation_sources: Vec<CitationSource>) -> String {
    let mut text = String::new();

    for source in citation_sources {
        if let Some(uri) = source.uri {
            if let Some(license) = source.license {
                if !license.is_empty() {
                    write!(text, "\n[{license}] {uri}").unwrap();
                }
            }

            text.push_str(&uri);
        }
    }

    text
}
