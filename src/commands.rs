use async_trait::async_trait;
use reqwest::StatusCode;
use tdlib::types::FormattedText;

use crate::bot::TdError;
use crate::utilities::api_utils::ServerError;
use crate::utilities::command_context::CommandContext;
use crate::utilities::convert_argument::ConversionError;
use crate::utilities::rate_limit::RateLimiter;

pub mod autocomplete;
pub mod badtranslate;
pub mod calculate_inline;
pub mod charinfo;
pub mod cobalt_download;
pub mod config;
pub mod craiyon;
pub mod craiyon_search;
pub mod delete;
pub mod dice_reply;
pub mod different_dimension_me;
pub mod google_palm;
pub mod kebab;
pub mod kiwifarms;
pub mod markov_chain;
pub mod ping;
pub mod radio_poligon;
pub mod screenshot;
pub mod sex;
pub mod stablehorde;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod trollslate;
pub mod urbandictionary;

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
    Custom(String),
    CustomFormattedText(FormattedText),
    ArgumentConversion(ConversionError),
    Telegram(TdError),
    Server(StatusCode),
    Reqwest(reqwest::Error),
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::Custom(value.into())
    }
}

impl From<ConversionError> for CommandError {
    fn from(value: ConversionError) -> Self {
        Self::ArgumentConversion(value)
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
