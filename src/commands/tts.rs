use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::types::{InputFileLocal, InputMessageVoiceNote};
use tempfile::NamedTempFile;

use super::CommandError::MissingArgument;
use super::{CommandResult, CommandTrait};
use crate::apis::ivona;
use crate::utils::Context;

#[derive(Default)]
pub struct Tts;

#[async_trait]
impl CommandTrait for Tts {
    fn name(&self) -> &'static str {
        "tts"
    }

    fn aliases(&self) -> &[&str] {
        &["ivona"]
    }

    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult {
        let text = arguments.ok_or(MissingArgument("text to synthesize"))?;

        if text.chars().count() > 1024 {
            Err("this text is too long.")?;
        }

        let bytes = ivona::synthesize(ctx.http_client.clone(), text, "jan").await?;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&bytes).unwrap();

        ctx.reply_custom(
            InputMessageContent::InputMessageVoiceNote(InputMessageVoiceNote {
                voice_note: InputFile::Local(InputFileLocal {
                    path: temp_file.path().to_str().unwrap().into(),
                }),
                duration: 0,
                waveform: String::new(),
                caption: None,
            }),
            None,
        )
        .await?;

        Ok(())
    }
}
