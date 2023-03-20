use std::borrow::Cow;
use std::fmt;

use async_trait::async_trait;
use tdlib::enums::Message;
use tdlib::functions;

use super::command_context::CommandContext;
use super::google_translate::LANGUAGES;
use super::telegram_utils;

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionError {
    MissingArgument,
    BadArgument(&'static str),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingArgument => write!(f, "missing command argument"),
            Self::BadArgument(reason) => write!(f, "bad command argument: {reason}"),
        }
    }
}

#[async_trait]
pub trait ConvertArgument: Sized + Send {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError>;
}

#[async_trait]
impl ConvertArgument for String {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let mut arguments = arguments.chars();
        let argument = arguments
            .by_ref()
            .skip_while(char::is_ascii_whitespace)
            .take_while(|char| !char.is_ascii_whitespace())
            .collect::<String>();

        if argument.is_empty() {
            Err(ConversionError::MissingArgument)?;
        }

        Ok((argument, arguments.as_str()))
    }
}

#[async_trait]
impl<T: ConvertArgument> ConvertArgument for Option<T> {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
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
    ) -> Result<(Self, &'a str), ConversionError> {
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
    ) -> Result<(Self, &'a str), ConversionError> {
        if ctx.message.reply_to_message_id == 0 {
            Err(ConversionError::MissingArgument)?;
        }

        let Message::Message(message) = functions::get_message(
            ctx.message.reply_in_chat_id,
            ctx.message.reply_to_message_id,
            ctx.client_id,
        )
        .await
        .map_err(|_| ConversionError::BadArgument("replied message cannot be loaded."))?;

        let argument = telegram_utils::get_message_text(&message)
            .ok_or(ConversionError::BadArgument("replied message doesn't contain any text."))?
            .text
            .clone();

        Ok((Self(argument), arguments))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringGreedy(pub String);

#[async_trait]
impl ConvertArgument for StringGreedy {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let argument = arguments.trim_start().to_owned();

        if argument.is_empty() {
            Err(ConversionError::MissingArgument)?;
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
    ) -> Result<(Self, &'a str), ConversionError> {
        match Option::<StringGreedy>::convert(ctx, arguments).await? {
            (Some(argument), arguments) => Ok((Self(argument.0), arguments)),
            (None, arguments) => {
                let (Reply(argument), arguments) = ConvertArgument::convert(ctx, arguments).await?;
                Ok((Self(argument), arguments))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Language(pub &'static str);

#[async_trait]
impl ConvertArgument for Language {
    async fn convert<'a>(
        _: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let arguments = arguments.trim_start();

        if arguments.is_empty() {
            Err(ConversionError::MissingArgument)?;
        }

        let lowercase = arguments.to_ascii_lowercase();

        for (language_code, language) in LANGUAGES {
            for prefix in [language_code, &language.to_ascii_lowercase()] {
                if lowercase.starts_with(prefix) {
                    let rest = &arguments[prefix.len()..];
                    if rest.chars().next().map_or(true, |char| char.is_ascii_whitespace()) {
                        return Ok((Self(language_code), rest));
                    }
                }
            }
        }

        Err(ConversionError::BadArgument("unknown language code or name."))
    }
}

pub struct SourceTargetLanguages(pub Option<&'static str>, pub Cow<'static, str>);

#[async_trait]
impl ConvertArgument for SourceTargetLanguages {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utilities::test_fixtures;

    #[tokio::test]
    async fn test_string_converter() {
        let ctx = test_fixtures::command_context();

        let result = String::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (argument, arguments) = String::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(arguments, "");

        let (argument, arguments) = String::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(arguments, "bar");

        let (argument, arguments) = String::convert(&ctx, " foo bar ").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(arguments, "bar ");

        let (argument, arguments) = String::convert(&ctx, "foo  bar").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(arguments, " bar");
    }

    #[tokio::test]
    async fn test_option_converter() {
        let ctx = test_fixtures::command_context();

        let (argument, arguments) = Option::<String>::convert(&ctx, "").await.unwrap();
        assert_eq!(argument, None);
        assert_eq!(arguments, "");

        let (argument, arguments) = Option::<String>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, Some("foo".into()));
        assert_eq!(arguments, "bar");
    }

    #[tokio::test]
    async fn test_multiple_converters() {
        let ctx = test_fixtures::command_context();

        let result = <(String, String)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(String, String)>::convert(&ctx, "foo").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (argument, arguments) = <(String, String)>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, ("foo".into(), "bar".into()));
        assert_eq!(arguments, "");

        let (argument, arguments) = <(String, String)>::convert(&ctx, "foo bar baz").await.unwrap();
        assert_eq!(argument, ("foo".into(), "bar".into()));
        assert_eq!(arguments, "baz");
    }

    #[tokio::test]
    async fn test_multiple_option_converters() {
        let ctx = test_fixtures::command_context();

        let (argument, arguments) =
            <(Option<String>, Option<String>)>::convert(&ctx, "").await.unwrap();
        assert_eq!(argument, (None, None));
        assert_eq!(arguments, "");

        let result = <(Option<String>, String)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(String, Option<String>)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(Option<String>, String)>::convert(&ctx, "foo").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (argument, arguments) = <(String, Option<String>)>::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, ("foo".into(), None));
        assert_eq!(arguments, "");

        let (argument, arguments) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), None));
        assert_eq!(arguments, "");

        let (argument, arguments) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), Some("bar".into())));
        assert_eq!(arguments, "");

        let (argument, arguments) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo bar baz").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), Some("bar".into())));
        assert_eq!(arguments, "baz");
    }

    #[tokio::test]
    async fn test_string_greedy_converter() {
        let ctx = test_fixtures::command_context();

        let result = StringGreedy::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (StringGreedy(argument), arguments) =
            ConvertArgument::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(arguments, "");

        let (StringGreedy(argument), arguments) =
            ConvertArgument::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, "foo bar");
        assert_eq!(arguments, "");

        let (StringGreedy(argument), arguments) =
            ConvertArgument::convert(&ctx, " foo bar ").await.unwrap();
        assert_eq!(argument, "foo bar ");
        assert_eq!(arguments, "");

        let (StringGreedy(argument), arguments) =
            ConvertArgument::convert(&ctx, "foo  bar").await.unwrap();
        assert_eq!(argument, "foo  bar");
        assert_eq!(arguments, "");
    }

    #[tokio::test]
    async fn test_language_converter() {
        let ctx = test_fixtures::command_context();

        let result = Language::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <Language>::convert(&ctx, "foo").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let (Language(argument), arguments) = ConvertArgument::convert(&ctx, "en").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(arguments, "");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "en foo").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(arguments, " foo");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "english").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(arguments, "");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "english FOO").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(arguments, " FOO");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "ENGLISH foo").await.unwrap();
        assert_eq!(argument, "en");
        assert_eq!(arguments, " foo");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "chinese (simplified)").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(arguments, "");

        let result = <Language>::convert(&ctx, "chinese").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let result = <Language>::convert(&ctx, "chinese  (simplified)").await;
        let Err(ConversionError::BadArgument(_)) = result else {
            panic!("expected BadArgument error");
        };

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "chinese (simplified) FOO").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(arguments, " FOO");

        let (Language(argument), arguments) =
            ConvertArgument::convert(&ctx, "CHINESE (SIMPLIFIED) foo").await.unwrap();
        assert_eq!(argument, "zh-cn");
        assert_eq!(arguments, " foo");
    }

    #[tokio::test]
    async fn test_source_target_languages_converter() {
        let ctx = test_fixtures::command_context();

        let (SourceTargetLanguages(source_language, target_language), arguments) =
            ConvertArgument::convert(&ctx, "").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "user_language_code");
        assert_eq!(arguments, "");

        let (SourceTargetLanguages(source_language, target_language), arguments) =
            ConvertArgument::convert(&ctx, "en").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "en");
        assert_eq!(arguments, "");

        let (SourceTargetLanguages(source_language, target_language), arguments) =
            ConvertArgument::convert(&ctx, "en foo").await.unwrap();
        assert_eq!(source_language, None);
        assert_eq!(target_language, "en");
        assert_eq!(arguments, " foo");

        let (SourceTargetLanguages(source_language, target_language), arguments) =
            ConvertArgument::convert(&ctx, "chinese (simplified) english").await.unwrap();
        assert_eq!(source_language, Some("zh-cn"));
        assert_eq!(target_language, "en");
        assert_eq!(arguments, "");

        let (SourceTargetLanguages(source_language, target_language), arguments) =
            ConvertArgument::convert(&ctx, "chinese (simplified) english foo").await.unwrap();
        assert_eq!(source_language, Some("zh-cn"));
        assert_eq!(target_language, "en");
        assert_eq!(arguments, " foo");
    }
}
