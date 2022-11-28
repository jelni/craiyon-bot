use std::convert::TryInto;

use tdlib::enums::TextEntityType;
use tdlib::types::FormattedText;

pub struct ParsedCommand {
    pub name: String,
    pub bot_username: Option<String>,
    pub arguments: Option<String>,
}

impl ParsedCommand {
    pub fn parse(formatted_text: &FormattedText) -> Option<ParsedCommand> {
        let entity = formatted_text
            .entities
            .iter()
            .find(|e| e.r#type == TextEntityType::BotCommand && e.offset == 0)?;

        let command = formatted_text
            .text
            .chars()
            .skip((entity.offset + 1).try_into().ok()?)
            .take((entity.length - 1).try_into().ok()?)
            .collect::<String>();

        let (command_name, username) = match command.split_once('@') {
            Some(parts) => (parts.0.into(), Some(parts.1)),
            None => (command, None),
        };

        let arguments = formatted_text
            .text
            .chars()
            .skip(entity.length.try_into().unwrap_or_default())
            .skip_while(char::is_ascii_whitespace)
            .collect::<String>();

        let arguments = if arguments.is_empty() { None } else { Some(arguments) };

        Some(ParsedCommand {
            name: command_name.to_lowercase(),
            bot_username: username.map(str::to_string),
            arguments,
        })
    }
}
