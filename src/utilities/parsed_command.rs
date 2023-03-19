use tdlib::enums::TextEntityType;
use tdlib::types::FormattedText;

pub struct ParsedCommand {
    pub name: String,
    pub bot_username: Option<String>,
    pub arguments: String,
}

impl ParsedCommand {
    pub fn parse(formatted_text: &FormattedText) -> Option<ParsedCommand> {
        let entity = formatted_text
            .entities
            .iter()
            .find(|e| e.r#type == TextEntityType::BotCommand && e.offset == 0)?;

        let command_name_range = {
            let start = entity.offset as usize + 1;
            start..entity.length as usize
        };

        let command = &formatted_text.text[command_name_range.clone()];

        let (command_name, username) = match command.split_once('@') {
            Some(parts) => (parts.0, Some(parts.1)),
            None => (command, None),
        };

        let arguments = formatted_text.text[command_name_range.end..].trim_start().into();

        Some(ParsedCommand {
            name: command_name.to_lowercase(),
            bot_username: username.map(str::to_string),
            arguments,
        })
    }
}
