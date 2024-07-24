# Craiyon Bot

## Overview

Craiyon Bot is a versatile Telegram bot implemented in Rust that provides a wide range of functionalities, including image generation, translation, jokes, and more. Initially focused on generating images using the Craiyon model, the bot has since expanded to include numerous features, making it a comprehensive tool for users on Telegram. The bot is designed to be modular, with each command encapsulated in its own module, allowing for easy maintenance and extensibility.

## Features

- **Image Generation**: Generate images using various models such as Craiyon and Stable Diffusion.
- **Translation**: Translate text using Google Translate and provide bad translations for fun.
- **Jokes**: Fetch jokes from different APIs.
- **Radio Streaming**: Get information about currently playing radio stations.
- **Urban Dictionary**: Fetch definitions from Urban Dictionary.
- **Markov Chain**: Generate text based on previous chat messages.
- **Command Autocomplete**: Suggest completions for user queries using Google.
- **File Downloading**: Download media files from various sources.
- **Interactive Commands**: Engage users with interactive commands that provide real-time responses and updates.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)
- Telegram Bot Token (from [BotFather](https://core.telegram.org/bots#botfather))
- API keys for external services (e.g., Google APIs, Stable Horde)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/craiyon-bot.git
   cd craiyon-bot
   ```

2. Create a `.env` file in the root directory and add your API keys and bot token:

   ```plaintext
   TELEGRAM_TOKEN=your_telegram_bot_token
   API_ID=your_api_id
   API_HASH=your_api_hash
   STABLEHORDE_CLIENT=your_stablehorde_client
   STABLEHORDE_TOKEN=your_stablehorde_token
   MAKERSUITE_API_KEY=your_makersuite_api_key
   ```

3. Build the project:

   ```bash
   cargo build
   ```

4. Run the bot:

   ```bash
   cargo run
   ```

### Usage

Once the bot is running, you can interact with it on Telegram. Here are some example commands you can use:

- `/start`: Get a welcome message and instructions on how to use the bot.
- `/craiyon_art <prompt>`: Generate an image based on the provided prompt using the Craiyon model.
- `/translate <text>`: Translate the provided text using Google Translate.
- `/moveit_joke`: Get a random joke from the MoveIt API.
- `/radio_poligon`: Get information about the currently playing radio station.

## Command Structure

The bot's commands are organized in the `commands` module. Each command implements the `CommandTrait`, which requires the following methods:

- `command_names`: Returns the names of the command.
- `description`: Provides a brief description of the command.
- `execute`: Contains the logic for executing the command.

### Example Command

Here’s an example of how a command is structured:

```rust
pub struct Start;

#[async_trait]
impl CommandTrait for Start {
    fn command_names(&self) -> &[&str] {
        &["start"]
    }

    async fn execute(&self, ctx: &CommandContext, _: String) -> CommandResult {
        ctx.reply("Welcome to the Craiyon Bot! Use /help to see available commands.").await?;
        Ok(())
    }
}
```

## API Integrations

The bot integrates with various APIs to provide its functionalities. Here are some of the key integrations:

- **Craiyon API**: For generating images based on user prompts.
- **Google Translate API**: For translating text between languages.
- **Urban Dictionary API**: For fetching definitions of words.
- **Stable Horde API**: For generating images using Stable Diffusion models.
- **MoveIt API**: For fetching jokes.

Each API integration is encapsulated in its own module under the `apis` directory, allowing for easy updates and maintenance.

## Logging

The bot uses a custom logging setup to log messages to a file. The log file is created in the root directory of the project and can be used for debugging and monitoring the bot's activity.

## Testing

The bot includes unit tests for various functionalities. You can run the tests using:

```bash
cargo test
```

## Contributing

Contributions are welcome! If you have suggestions or improvements, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
