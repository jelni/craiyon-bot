use std::sync::Arc;

use async_trait::async_trait;

use crate::ratelimit::RateLimiter;
use crate::utils::Context;

pub mod autocomplete;
pub mod badtranslate;
pub mod charinfo;
pub mod cobalt_download;
pub mod delete;
pub mod generate;
pub mod kiwifarms;
pub mod ping;
pub mod screenshot;
pub mod sex;
pub mod stable_horde;
pub mod start;
pub mod startit_joke;
pub mod translate;
pub mod tts;
pub mod urbandictionary;

pub type CommandResult = Result<(), CommandError>;

#[async_trait]
pub trait CommandTrait {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &[&str] {
        &[]
    }
    fn rate_limit(&self) -> RateLimiter<i64> {
        RateLimiter::new(4, 20)
    }
    async fn execute(&self, ctx: Arc<Context>, arguments: Option<String>) -> CommandResult;
}

pub enum CommandError {
    CustomError(String),
    CustomMarkdownError(String),
    MissingArgument(&'static str),
    TelegramError(tgbotapi::Error),
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

impl From<tgbotapi::Error> for CommandError {
    fn from(value: tgbotapi::Error) -> Self {
        Self::TelegramError(value)
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}
