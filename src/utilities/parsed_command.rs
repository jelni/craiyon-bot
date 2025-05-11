use tdlib::enums::TextEntityType;
use tdlib::types::FormattedText;

pub struct ParsedCommand {
    pub name: String,
    pub bot_username: Option<String>,
    pub arguments: String,
}

impl ParsedCommand {
    pub fn parse(formatted_text: &FormattedText) -> Option<Self> {
        let entity = formatted_text
            .entities
            .iter()
            .find(|e| e.r#type == TextEntityType::BotCommand && e.offset == 0)?;

        #[expect(clippy::cast_sign_loss)]
        let command_name_range = {
            let start = entity.offset as usize + 1;
            start..entity.length as usize
        };

        let command = &formatted_text.text[command_name_range.clone()];

        let (command_name, username) =
            command.split_once('@').map_or((command, None), |parts| (parts.0, Some(parts.1)));

        let arguments = formatted_text.text[command_name_range.end..].trim_ascii_start().into();

        Some(Self {
            name: command_name.to_lowercase(),
            bot_username: username.map(str::to_string),
            arguments,
        })
    }
}
