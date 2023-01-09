use std::fmt;
use std::sync::{Arc, Mutex};

use tdlib::types::BotCommand;

use super::rate_limit::RateLimiter;
use crate::commands::CommandTrait;

pub type CommandRef = Box<dyn CommandTrait + Send + Sync>;

pub struct CommandInstance {
    pub command: CommandRef,
    pub rate_limiter: Mutex<RateLimiter<i64>>,
}

impl fmt::Display for CommandInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.command.command_names().first().unwrap())
    }
}

pub struct CommandManager {
    commands: Vec<Arc<CommandInstance>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn add_command(&mut self, command: CommandRef) {
        self.commands.push(Arc::new(CommandInstance {
            rate_limiter: Mutex::new(command.rate_limit()),
            command,
        }));
    }

    pub fn get_command(&self, name: &str) -> Option<Arc<CommandInstance>> {
        self.commands.iter().find(|c| c.command.command_names().contains(&name)).cloned()
    }

    pub fn public_command_list(&self) -> Vec<BotCommand> {
        self.commands
            .iter()
            .filter_map(|c| {
                c.command.description().map(|d| BotCommand {
                    command: (*c.command.command_names().first().unwrap()).into(),
                    description: d.into(),
                })
            })
            .collect()
    }
}
