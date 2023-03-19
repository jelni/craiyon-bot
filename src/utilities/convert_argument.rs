use std::borrow::Cow;

use async_trait::async_trait;
use tdlib::enums::Message;
use tdlib::functions;

use super::command_context::CommandContext;
use super::google_translate::LANGUAGES;
use super::telegram_utils;
use crate::commands::CommandError;

#[async_trait]
pub trait ConvertArgument: Sized + Send {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError>;
}

#[async_trait]
impl ConvertArgument for String {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        let mut arguments = arguments.chars();
        let argument = arguments
            .by_ref()
            .skip_while(char::is_ascii_whitespace)
            .take_while(|char| !char.is_ascii_whitespace())
            .collect::<String>();

        if argument.is_empty() {
            Err(CommandError::MissingArgument)?;
        }

        Ok((argument, arguments.as_str()))
    }
}

#[async_trait]
impl<T: ConvertArgument> ConvertArgument for Option<T> {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        match T::convert(ctx, arguments).await {
            Ok((argument, arguments)) => Ok((Some(argument), arguments)),
            Err(_) => Ok((None, arguments)),
        }
    }
}

#[async_trait]
impl<T1, T2> ConvertArgument for (T1, T2)
where
    T1: ConvertArgument,
    T2: ConvertArgument,
{
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        let (arg1, arguments) = T1::convert(ctx, arguments).await?;
        let (arg2, arguments) = T2::convert(ctx, arguments).await?;

        Ok(((arg1, arg2), arguments))
    }
}

pub struct Reply(pub String);

#[async_trait]
impl ConvertArgument for Reply {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        if ctx.message.reply_to_message_id == 0 {
            Err(CommandError::MissingArgument)?;
        }

        let Message::Message(message) = functions::get_message(
            ctx.message.reply_in_chat_id,
            ctx.message.reply_to_message_id,
            ctx.client_id,
        )
        .await?;

        let argument = telegram_utils::get_message_text(&message)
            .ok_or(CommandError::ArgumentParseError(
                "replied message doesn't contain any text.".into(),
            ))?
            .text
            .clone();

        Ok((Self(argument), arguments))
    }
}

pub struct StringGreedy(pub String);

#[async_trait]
impl ConvertArgument for StringGreedy {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        let argument = arguments.trim_start().to_owned();

        if argument.is_empty() {
            Err(CommandError::MissingArgument)?;
        }

        Ok((Self(argument), ""))
    }
}

pub struct StringGreedyOrReply(pub String);

#[async_trait]
impl ConvertArgument for StringGreedyOrReply {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        match Option::<StringGreedy>::convert(ctx, arguments).await? {
            (Some(argument), arguments) => Ok((Self(argument.0), arguments)),
            (None, arguments) => {
                let (Reply(argument), arguments) = ConvertArgument::convert(ctx, arguments).await?;
                Ok((Self(argument), arguments))
            }
        }
    }
}

pub struct Language(pub &'static str);

#[async_trait]
impl ConvertArgument for Language {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        let lowercase = arguments.to_ascii_lowercase();
        let words = lowercase.split_ascii_whitespace().collect::<Vec<_>>();

        if words.is_empty() {
            Err(CommandError::MissingArgument)?;
        }

        for (language_code, language) in LANGUAGES {
            for prefix in [language_code, &language.to_ascii_lowercase()] {
                if words.starts_with(&prefix.split_ascii_whitespace().collect::<Vec<_>>()) {
                    return Ok((Self(language_code), &arguments[prefix.len()..]));
                }
            }
        }

        Err(CommandError::ArgumentParseError("unknown language code or name.".into()))
    }
}

pub struct SourceTargetLanguages(pub Option<&'static str>, pub Cow<'static, str>);

#[async_trait]
impl ConvertArgument for SourceTargetLanguages {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), CommandError> {
        let Some((Language(first_language), arguments)) =
            Language::convert(ctx, arguments).await.ok()
        else {
            let target_language = if ctx.user.language_code.is_empty() {
                Cow::Borrowed("en")
            } else {
                Cow::Owned(ctx.user.language_code.clone())
            };

            return Ok((SourceTargetLanguages(None, target_language), arguments));
        };

        let Some((Language(second_language), arguments)) =
            Language::convert(ctx, arguments).await.ok()
        else {
            return Ok((SourceTargetLanguages(None, Cow::Borrowed(first_language)), arguments));
        };

        Ok((SourceTargetLanguages(Some(first_language), Cow::Borrowed(second_language)), arguments))
    }
}
