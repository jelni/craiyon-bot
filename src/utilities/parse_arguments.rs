use async_trait::async_trait;

use super::command_context::CommandContext;
use super::convert_argument::ConvertArgument;
use crate::commands::CommandError;

#[async_trait]
pub trait ParseArguments: Sized {
    async fn parse_arguments(ctx: &CommandContext, arguments: &str) -> Result<Self, CommandError>;
}

#[async_trait]
impl<T1> ParseArguments for T1
where
    T1: ConvertArgument,
{
    async fn parse_arguments(ctx: &CommandContext, arguments: &str) -> Result<Self, CommandError> {
        let arguments = arguments.chars();

        let (arg1, _) = T1::convert(ctx, arguments).await?;

        Ok(arg1)
    }
}

#[async_trait]
impl<T1, T2> ParseArguments for (T1, T2)
where
    T1: ConvertArgument,
    T2: ConvertArgument,
{
    async fn parse_arguments(ctx: &CommandContext, arguments: &str) -> Result<Self, CommandError> {
        let arguments = arguments.chars();

        let (arg1, arguments) = T1::convert(ctx.clone(), arguments).await?;
        let (arg2, _) = T2::convert(ctx, arguments).await?;

        Ok((arg1, arg2))
    }
}
