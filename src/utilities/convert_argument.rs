use std::borrow::Cow;
use std::fmt;

use async_trait::async_trait;
use tdlib::enums::{Message, MessageReplyTo};
use tdlib::functions;

use super::command_context::CommandContext;
use super::telegram_utils;

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionError {
    MissingArgument,
    BadArgument(&'static str),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        let mut arguments = arguments.trim_start().chars();
        let argument =
            arguments.by_ref().take_while(|char| !char.is_ascii_whitespace()).collect::<Self>();

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
            Ok((argument, rest)) => Ok((Some(argument), rest)),
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
        let (arg1, rest) = T1::convert(ctx, arguments).await?;
        let (arg2, rest) = T2::convert(ctx, rest).await?;

        Ok(((arg1, arg2), rest))
    }
}

pub struct Reply(pub String);

#[async_trait]
impl ConvertArgument for Reply {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let Some(MessageReplyTo::Message(reply)) = &ctx.message.reply_to else {
            return Err(ConversionError::MissingArgument);
        };

        if let Some(quote) = reply.quote.as_ref() {
            return Ok((Self(quote.text.text.clone()), arguments));
        };

        let content = if let Some(content) = reply.content.as_ref() {
            Cow::Borrowed(content)
        } else {
            let Message::Message(message) =
                functions::get_replied_message(ctx.message.chat_id, ctx.message.id, ctx.client_id)
                    .await
                    .map_err(|_| {
                        ConversionError::BadArgument("replied message couldn't be loaded.")
                    })?;

            Cow::Owned(message.content)
        };

        let argument = telegram_utils::get_message_text(&content)
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
            (Some(argument), rest) => Ok((Self(argument.0), rest)),
            (None, rest) => {
                let (Reply(argument), _): (Reply, _) = ConvertArgument::convert(ctx, rest).await?;
                let (argument, _) = StringGreedy::convert(ctx, &argument).await?;
                Ok((Self(argument.0), ""))
            }
        }
    }
}

#[async_trait]
impl ConvertArgument for bool {
    async fn convert<'a>(
        ctx: &CommandContext,
        arguments: &'a str,
    ) -> Result<(Self, &'a str), ConversionError> {
        let (mut argument, rest) = String::convert(ctx, arguments).await?;
        argument.make_ascii_lowercase();

        let value = if ["true", "yes", "on", "enable", "enabled"].contains(&argument.as_str()) {
            true
        } else if ["false", "no", "off", "disable", "disabled"].contains(&argument.as_str()) {
            false
        } else {
            return Err(ConversionError::BadArgument("argument cannot be converted to a bool."));
        };

        Ok((value, rest))
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

        let (argument, rest) = String::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(rest, "");

        let (argument, rest) = String::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(rest, "bar");

        let (argument, rest) = String::convert(&ctx, " foo bar ").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(rest, "bar ");

        let (argument, rest) = String::convert(&ctx, "foo  bar").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(rest, " bar");
    }

    #[tokio::test]
    async fn test_option_converter() {
        let ctx = test_fixtures::command_context();

        let (argument, rest) = Option::<String>::convert(&ctx, "").await.unwrap();
        assert_eq!(argument, None);
        assert_eq!(rest, "");

        let (argument, rest) = Option::<String>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, Some("foo".into()));
        assert_eq!(rest, "bar");
    }

    #[tokio::test]
    async fn test_multiple_converters() {
        let ctx = test_fixtures::command_context();

        let result = <(String, String)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(String, String)>::convert(&ctx, "foo").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (argument, rest) = <(String, String)>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, ("foo".into(), "bar".into()));
        assert_eq!(rest, "");

        let (argument, rest) = <(String, String)>::convert(&ctx, "foo bar baz").await.unwrap();
        assert_eq!(argument, ("foo".into(), "bar".into()));
        assert_eq!(rest, "baz");
    }

    #[tokio::test]
    async fn test_multiple_option_converters() {
        let ctx = test_fixtures::command_context();

        let (argument, rest) = <(Option<String>, Option<String>)>::convert(&ctx, "").await.unwrap();
        assert_eq!(argument, (None, None));
        assert_eq!(rest, "");

        let result = <(Option<String>, String)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(String, Option<String>)>::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let result = <(Option<String>, String)>::convert(&ctx, "foo").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (argument, rest) = <(String, Option<String>)>::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, ("foo".into(), None));
        assert_eq!(rest, "");

        let (argument, rest) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), None));
        assert_eq!(rest, "");

        let (argument, rest) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), Some("bar".into())));
        assert_eq!(rest, "");

        let (argument, rest) =
            <(Option<String>, Option<String>)>::convert(&ctx, "foo bar baz").await.unwrap();
        assert_eq!(argument, (Some("foo".into()), Some("bar".into())));
        assert_eq!(rest, "baz");
    }

    #[tokio::test]
    async fn test_string_greedy_converter() {
        let ctx = test_fixtures::command_context();

        let result = StringGreedy::convert(&ctx, "").await;
        assert_eq!(result, Err(ConversionError::MissingArgument));

        let (StringGreedy(argument), rest) = ConvertArgument::convert(&ctx, "foo").await.unwrap();
        assert_eq!(argument, "foo");
        assert_eq!(rest, "");

        let (StringGreedy(argument), rest) =
            ConvertArgument::convert(&ctx, "foo bar").await.unwrap();
        assert_eq!(argument, "foo bar");
        assert_eq!(rest, "");

        let (StringGreedy(argument), rest) =
            ConvertArgument::convert(&ctx, " foo bar ").await.unwrap();
        assert_eq!(argument, "foo bar ");
        assert_eq!(rest, "");

        let (StringGreedy(argument), rest) =
            ConvertArgument::convert(&ctx, "foo  bar").await.unwrap();
        assert_eq!(argument, "foo  bar");
        assert_eq!(rest, "");
    }
}
