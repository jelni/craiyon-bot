use std::borrow::Cow;
use std::fmt::Write;
use std::time::Duration;

use async_trait::async_trait;
use tdlib::enums::File;
use tdlib::types::{FormattedText, Message};
use tdlib::{enums, functions};
use tokio::sync::mpsc;
use tokio::time::Instant;

use super::{CommandError, CommandResult, CommandTrait};
use crate::apis::google_aistudio::{
    self, Candidate, CitationSource, Content, FileData, GenerateContentResponse, Part, PartResponse,
};
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::{ConvertArgument, ReplyChain};
use crate::utilities::file_download::MEBIBYTE;
use crate::utilities::rate_limit::RateLimiter;
use crate::utilities::telegram_utils;

pub const SYSTEM_INSTRUCTION: &str =
    "Be concise and precise. Don't be verbose. Answer in the language the user wrote in.";

pub struct Gemini {
    command_names: &'static [&'static str],
    description: &'static str,
    model: &'static str,
}

impl Gemini {
    #[expect(clippy::self_named_constructors)]
    pub const fn gemini() -> Self {
        Self {
            command_names: &["gemini", "g"],
            description: "ask Gemini 1.5 Flash",
            model: "gemini-1.5-flash-latest",
        }
    }

    pub const fn gemini2() -> Self {
        Self {
            command_names: &["gemini2", "g2"],
            description: "ask Gemini 2.0 Flash",
            model: "gemini-2.0-flash-exp",
        }
    }
}

#[async_trait]
impl CommandTrait for Gemini {
    fn command_names(&self) -> &[&str] {
        self.command_names
    }

    fn description(&self) -> Option<&'static str> {
        Some(self.description)
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(6, 60)
    }

    #[expect(clippy::too_many_lines)]
    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult {
        let ReplyChain(messages) = ConvertArgument::convert(ctx, &arguments).await?.0;
        ctx.send_typing().await?;

        let mut contents = Vec::new();

        for message in messages {
            let mut parts = Vec::new();

            if let Some(text) = message.text {
                parts.push(Part::Text(Cow::Owned(text)));
            }

            if let Some(message_image) = telegram_utils::get_message_attachment(
                Cow::Owned(message.content),
                true,
                ctx.client_id,
            )
            .await?
            {
                let file = message_image.file()?;

                if file.size > 64 * MEBIBYTE {
                    return Err(CommandError::Custom("files cannot be larger than 64 MiB.".into()));
                }

                let File::File(file) =
                    functions::download_file(file.id, 1, 0, 0, true, ctx.client_id).await?;

                let open_file = tokio::fs::File::open(file.local.path).await.unwrap();

                let file = google_aistudio::upload_file(
                    &ctx.bot_state.http_client,
                    open_file,
                    file.size.try_into().unwrap(),
                    message_image.mime_type()?,
                )
                .await?;

                parts.push(Part::FileData(FileData { file_uri: file.uri }));
            }

            if !parts.is_empty() {
                contents.push(Content {
                    parts: Cow::Owned(parts),
                    role: Some(if message.my { "model" } else { "user" }),
                });
            }
        }

        if contents.is_empty() {
            return Err(CommandError::Custom("no prompt or file provided.".into()));
        }

        let system_instruction = if contents
            .iter()
            .any(|content| content.parts.iter().any(|part| matches!(part, Part::Text(..))))
        {
            Some(Content {
                parts: Cow::Borrowed([Part::Text(Cow::Borrowed(SYSTEM_INSTRUCTION))].as_slice()),
                role: None,
            })
        } else {
            contents.push(Content {
                parts: Cow::Borrowed(
                    [Part::Text(Cow::Borrowed("Comment briefly on what you see."))].as_slice(),
                ),
                role: Some("user"),
            });

            None
        };

        let http_client = ctx.bot_state.http_client.clone();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let model = self.model;

        tokio::spawn(async move {
            google_aistudio::stream_generate_content(
                http_client,
                tx,
                model,
                Cow::Owned(contents),
                system_instruction,
                512,
            )
            .await;
        });

        let mut last_update = Instant::now();
        let mut changed_after_last_update = false;
        let mut progress = Option::<GenerationProgress>::None;
        let mut message = Option::<Message>::None;

        loop {
            let (update_message, finished) = if let Ok(response) =
                tokio::time::timeout_at(last_update + Duration::from_secs(5), rx.recv()).await
            {
                match response {
                    Some(response) => {
                        let response = response?;

                        match progress.as_mut() {
                            Some(progress) => {
                                progress.update(response)?;
                                changed_after_last_update = true;
                            }
                            None => {
                                if let Some(candidate) = response.candidates.into_iter().next() {
                                    progress = Some(GenerationProgress::new(candidate));
                                    changed_after_last_update = true;
                                }
                            }
                        }

                        (false, false)
                    }
                    None => (true, true),
                }
            } else {
                last_update = Instant::now();
                (true, false)
            };

            if update_message && changed_after_last_update {
                let text = match progress.as_ref() {
                    Some(progress) => progress.format(finished),
                    None => {
                        continue;
                    }
                };

                let enums::FormattedText::FormattedText(formatted_text) =
                    functions::parse_markdown(
                        FormattedText { text, ..Default::default() },
                        ctx.client_id,
                    )
                    .await?;

                if let Some(message) = message.as_ref() {
                    ctx.edit_message_formatted_text(message.id, formatted_text).await?;
                } else {
                    let unsent_message = ctx.reply_formatted_text(formatted_text).await?;
                    message = Some(
                        ctx.bot_state.message_queue.wait_for_message(unsent_message.id).await?,
                    );
                }

                last_update = Instant::now();
                changed_after_last_update = false;
            }

            if finished {
                break;
            }
        }

        Ok(())
    }
}
struct GenerationProgress {
    parts: Vec<PartResponse>,
    finish_reason: Option<String>,
    citation_sources: Vec<CitationSource>,
}

impl GenerationProgress {
    fn new(candidate: Candidate) -> Self {
        Self {
            parts: candidate.content.map(|content| content.parts).unwrap_or_default(),
            finish_reason: candidate.finish_reason,
            citation_sources: candidate
                .citation_metadata
                .map(|citation_metadata| citation_metadata.citation_sources)
                .unwrap_or_default(),
        }
    }

    fn update(&mut self, response: GenerateContentResponse) -> Result<(), CommandError> {
        if let Some(prompt_feedback) = response.prompt_feedback {
            if let Some(block_reason) = &prompt_feedback.block_reason {
                if block_reason == "SAFETY" {
                    if let Some(safety_ratings) = &prompt_feedback.safety_ratings {
                        let reasons = safety_ratings
                            .iter()
                            .filter(|safety_rating| safety_rating.blocked)
                            .map(|safety_rating| safety_rating.category.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");

                        return Err(CommandError::Custom(format!(
                            "request blocked by Google: {reasons}."
                        )));
                    }
                }

                return Err(CommandError::Custom("request blocked by Google.".into()));
            }
        }

        let Some(candidate) = response.candidates.into_iter().next() else {
            return Err(CommandError::Custom("no response generated.".into()));
        };

        if let Some(content) = candidate.content {
            self.parts.extend(content.parts);

            self.citation_sources = candidate
                .citation_metadata
                .map(|citation_metadata| citation_metadata.citation_sources)
                .unwrap_or_default();
        }

        self.finish_reason = candidate.finish_reason;

        Ok(())
    }

    fn format(&self, finished: bool) -> String {
        let mut text = self
            .parts
            .iter()
            .map(|part| match part {
                PartResponse::Text(text) => text.as_str(),
                PartResponse::InlineData => "[unsupported response part]",
            })
            .collect::<Vec<_>>()
            .concat();

        if !finished {
            text.push('…');
        }

        if let Some(finish_reason) = self.finish_reason.as_ref() {
            if finish_reason != "STOP" {
                write!(text, " [finish reason: {finish_reason}]").unwrap();
            }
        }

        if !self.citation_sources.is_empty() {
            text.push_str("\n\n");
            text.push_str(&format_citations(&self.citation_sources));
        }

        text
    }
}

fn format_citations(citation_sources: &[CitationSource]) -> String {
    let mut text = String::new();

    for (i, source) in citation_sources.iter().enumerate() {
        if let Some(uri) = source.uri.as_ref() {
            write!(text, "\n[{}] ", i + 1).unwrap();

            if let Some(license) = source.license.as_ref() {
                if !license.is_empty() {
                    write!(text, "[{license}] ").unwrap();
                }
            }

            text.push_str(uri);
        }
    }

    text
}
