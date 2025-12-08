use std::borrow::Cow;

use async_trait::async_trait;
use reqwest::StatusCode;
use tdlib::types::FormattedText;

use crate::apis::google_aistudio::GenerationError;
use crate::bot::TdError;
use crate::utilities;
use crate::utilities::api_utils::ServerError;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::ConversionError;
use crate::utilities::file_download::DownloadError;
use crate::utilities::rate_limit::RateLimiter;

pub mod autocomplete;
pub mod badtranslate;
pub mod calculate_inline;
pub mod charinfo;
pub mod cobalt_download;
pub mod config;
pub mod convert;
pub mod delete;
pub mod dice_reply;
pub mod different_dimension_me;
pub mod fal;
pub mod gemini;
pub mod groq;
pub mod kebab;
pub mod kiwifarms;
pub mod markov_chain;
pub mod mevo;
pub mod moveit_joke;
pub mod openrouter;
pub mod petittube;
pub mod ping;
pub mod polymarket;
pub mod radio_poligon;
pub mod radio_sur;
pub mod screenshot;
pub mod sex;
pub mod stablehorde;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod trollslate;
pub mod urbandictionary;
pub mod yt_dlp;

pub type CommandResult = Result<(), CommandError>;

#[async_trait]
pub trait CommandTrait {
    fn command_names(&self) -> &[&str];

    fn description(&self) -> Option<&'static str> {
        None
    }

    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(3, 30)
    }

    async fn execute(&self, ctx: &CommandContext, arguments: String) -> CommandResult;
}

#[derive(Debug)]
pub enum CommandError {
    Custom(Cow<'static, str>),
    CustomFormattedText(FormattedText),
    ArgumentConversion(ConversionError),
    Telegram(TdError),
    Server(StatusCode),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    Download(DownloadError),
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::Custom(Cow::Owned(value))
    }
}

impl From<&'static str> for CommandError {
    fn from(value: &'static str) -> Self {
        Self::Custom(Cow::Borrowed(value))
    }
}

impl From<FormattedText> for CommandError {
    fn from(value: FormattedText) -> Self {
        Self::CustomFormattedText(value)
    }
}

impl From<ConversionError> for CommandError {
    fn from(value: ConversionError) -> Self {
        Self::ArgumentConversion(value)
    }
}

impl From<GenerationError> for CommandError {
    fn from(value: GenerationError) -> Self {
        match value {
            GenerationError::Network(err) => Self::Reqwest(err),
            GenerationError::Google(err) => {
                if err.iter().any(|error| error.code == StatusCode::TOO_MANY_REQUESTS.as_u16()) {
                    Self::Custom(Cow::Borrowed("[rate limit]"))
                } else {
                    Self::Custom(Cow::Owned(
                        err.into_iter()
                            .map(|error| error.to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ))
                }
            }
        }
    }
}

impl From<TdError> for CommandError {
    fn from(value: TdError) -> Self {
        Self::Telegram(value)
    }
}

impl From<ServerError> for CommandError {
    fn from(value: ServerError) -> Self {
        Self::Server(value.0)
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl From<DownloadError> for CommandError {
    fn from(value: DownloadError) -> Self {
        Self::Download(value)
    }
}

impl From<utilities::yt_dlp::Error> for CommandError {
    fn from(value: utilities::yt_dlp::Error) -> Self {
        match value {
            utilities::yt_dlp::Error::YtDlp(error) => Self::Custom(Cow::Owned(error)),
            utilities::yt_dlp::Error::Serde(error) => Self::SerdeJson(error),
        }
    }
}
