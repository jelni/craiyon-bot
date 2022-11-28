use std::sync::Arc;

use async_trait::async_trait;

use crate::bot::TdError;
use crate::command_context::CommandContext;
use crate::ratelimit::RateLimiter;

pub mod autocomplete;
pub mod badtranslate;
pub mod charinfo;
pub mod cobalt_download;
pub mod delete;
pub mod generate;
pub mod kebab;
pub mod kiwifarms;
pub mod ping;
pub mod screenshot;
pub mod sex;
pub mod stablehorde;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod tts;
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

    async fn execute(&self, ctx: Arc<CommandContext>, arguments: Option<String>) -> CommandResult;
}

pub enum CommandError {
    CustomError(String),
    CustomMarkdownError(String),
    MissingArgument(&'static str),
    TelegramError(TdError),
    ReqwestError(reqwest::Error),
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::CustomError(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::CustomError(value.into())
    }
}

impl From<TdError> for CommandError {
    fn from(value: TdError) -> Self {
        Self::TelegramError(value)
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}
