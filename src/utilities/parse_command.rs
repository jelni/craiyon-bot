use async_trait::async_trait;

pub enum ParseError {
    MissingArgument,
    TooManyArguments,
}

#[async_trait]
pub trait ExecuteCommand {
    async fn execute(&self);
}

#[async_trait]
pub trait ParseCommand: Sized {
    async fn parse_command(command: &str) -> Result<Self, ParseError>;
}

#[async_trait]
pub trait ParseArgument: Sized {
    async fn parse_argument<'a, T: Iterator<Item = char> + Send + 'a>(
        arguments: T,
    ) -> Result<(Self, Box<T>), ParseError>;
}

#[async_trait]
impl ParseArgument for String {
    async fn parse_argument<'a, T: Iterator<Item = char> + Send + 'a>(
        mut arguments: T,
    ) -> Result<(Self, Box<T>), ParseError> {
        let argument =
            arguments.by_ref().take_while(|char| !char.is_ascii_whitespace()).collect::<Self>();

        if argument.is_empty() {
            Err(ParseError::MissingArgument)?;
        }

        Ok((argument, Box::new(arguments)))
    }
}
