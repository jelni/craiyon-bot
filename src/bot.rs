use std::env;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::{redirect, Client};
use tdlib::enums::{
    AuthorizationState, BotCommands, ConnectionState, MessageContent, MessageSender, OptionValue,
    Update, UserType,
};
use tdlib::functions;
use tdlib::types::{
    BotCommand, MessageSenderUser, OptionValueInteger, OptionValueString, UpdateAuthorizationState,
    UpdateChatMember, UpdateChatPermissions, UpdateChatTitle, UpdateConnectionState,
    UpdateMessageSendFailed, UpdateMessageSendSucceeded, UpdateNewChat, UpdateNewInlineQuery,
    UpdateNewMessage, UpdateOption, UpdateUser,
};
use tokio::signal;
use tokio::task::JoinHandle;

use crate::commands::{calculate_inline, dice_reply, CommandTrait};
use crate::utilities::cache::{Cache, CompactUser};
use crate::utilities::command_context::CommandContext;
use crate::utilities::command_manager::CommandManager;
use crate::utilities::message_queue::MessageQueue;
use crate::utilities::parsed_command::ParsedCommand;
use crate::utilities::rate_limit::{RateLimiter, RateLimits};
use crate::utilities::{command_dispatcher, telegram_utils};

pub type TdError = tdlib::types::Error;
pub type TdResult<T> = Result<T, TdError>;

#[derive(Clone, Copy)]
enum BotState {
    Running,
    WaitingToClose,
    Closing,
    Closed,
}

pub struct Bot {
    client_id: i32,
    state: Arc<Mutex<BotState>>,
    my_id: Option<i64>,
    cache: Cache,
    http_client: reqwest::Client,
    command_manager: CommandManager,
    message_queue: Arc<MessageQueue>,
    rate_limits: Arc<Mutex<RateLimits>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            client_id: tdlib::create_client(),
            state: Arc::new(Mutex::new(BotState::Closed)),
            my_id: None,
            cache: Cache::default(),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            command_manager: CommandManager::new(),
            rate_limits: Arc::new(Mutex::new(RateLimits {
                rate_limit_exceeded: RateLimiter::new(1, 20),
            })),
            tasks: Vec::new(),
            message_queue: Arc::new(MessageQueue::default()),
        }
    }

    pub async fn run(&mut self) {
        *self.state.lock().unwrap() = BotState::Running;
        let client_id = self.client_id;
        self.run_task(async move {
            functions::set_log_verbosity_level(1, client_id).await.unwrap();
        });

        let state = self.state.clone();
        self.run_task(async move {
            signal::ctrl_c().await.unwrap();
            log::warn!("Ctrl+C received");
            *state.lock().unwrap() = BotState::WaitingToClose;
        });

        let mut last_task_count = 0;
        loop {
            if let Some((update, _)) = tdlib::receive() {
                self.on_update(update);
            }
            self.tasks.retain(|t| !t.is_finished());
            let state = *self.state.lock().unwrap();
            match state {
                BotState::WaitingToClose => {
                    if self.tasks.is_empty() {
                        self.close();
                    } else {
                        let task_count = self.tasks.len();
                        if task_count != last_task_count {
                            log::info!("waiting for {task_count} task(s) to finish…");
                            last_task_count = task_count;
                        }
                    }
                }
                BotState::Closed => break,
                _ => (),
            }
        }
    }

    fn close(&mut self) {
        *self.state.lock().unwrap() = BotState::Closing;
        let client_id = self.client_id;
        self.run_task(async move {
            functions::close(client_id).await.unwrap();
        });
    }

    fn run_task<T: Future<Output = ()> + Send + 'static>(&mut self, future: T) {
        self.tasks.push(tokio::spawn(future));
    }

    fn on_update(&mut self, update: Update) {
        match update {
            Update::AuthorizationState(update) => self.on_authorization_state(update),
            Update::NewMessage(update) => self.on_new_message(update),
            Update::MessageSendSucceeded(update) => self.on_message_send_succeeded(update),
            Update::MessageSendFailed(update) => self.on_message_send_failed(update),
            Update::NewChat(update) => self.on_new_chat(update),
            Update::ChatTitle(update) => self.on_chat_title(update),
            Update::ChatPermissions(update) => self.on_chat_permissions(update),
            Update::User(update) => self.on_user(update),
            Update::Option(update) => self.on_option(update),
            Update::ConnectionState(update) => self.on_connection_state(&update),
            Update::NewInlineQuery(update) => self.on_new_inline_query(update),
            Update::ChatMember(update) => self.on_chat_member(update),
            _ => (),
        }
    }

    fn on_authorization_state(&mut self, update: UpdateAuthorizationState) {
        let authorization_state = update.authorization_state;
        log::info!("authorization: {authorization_state:?}");
        match authorization_state {
            AuthorizationState::WaitTdlibParameters => {
                let client_id = self.client_id;
                self.run_task(async move {
                    functions::set_tdlib_parameters(
                        false,
                        ".data".into(),
                        String::new(),
                        env::var("DB_ENCRYPTION_KEY").unwrap(),
                        true,
                        true,
                        false,
                        false,
                        env::var("API_ID").unwrap().parse().unwrap(),
                        env::var("API_HASH").unwrap(),
                        "en".into(),
                        env!("CARGO_PKG_NAME").into(),
                        String::new(),
                        env!("CARGO_PKG_VERSION").into(),
                        true,
                        true,
                        client_id,
                    )
                    .await
                    .unwrap();
                });
            }
            AuthorizationState::WaitPhoneNumber => {
                let client_id = self.client_id;
                self.run_task(async move {
                    functions::check_authentication_bot_token(
                        env::var("TELEGRAM_TOKEN").unwrap(),
                        client_id,
                    )
                    .await
                    .unwrap();
                });
            }
            AuthorizationState::Closed => *self.state.lock().unwrap() = BotState::Closed,
            _ => (),
        }
    }

    fn on_ready(&mut self) {
        let client_id = self.client_id;
        let commands = self.command_manager.public_command_list();
        self.run_task(async move {
            functions::get_me(client_id).await.unwrap();
            Bot::sync_commands(commands, client_id).await.unwrap();
        });
    }

    fn on_new_message(&mut self, update: UpdateNewMessage) {
        if update.message.forward_info.is_some() {
            return; // ignore forwarded messages
        }
        let MessageSender::User(MessageSenderUser { user_id }) = update.message.sender_id else {
            return; // ignore messages not sent by users
        };
        let Some(user) = self.cache.get_user(user_id) else {
            return; // ignore users not in cache
        };
        let UserType::Regular = user.r#type else {
            return; // ignore bots
        };
        let Some(chat) = self.cache.get_chat(update.message.chat_id) else {
            return; // ignore chats not in cache
        };
        if let MessageContent::MessageDice(_) = update.message.content {
            self.run_task(dice_reply::execute(update.message, self.client_id));
            return;
        }
        let Some(text) = telegram_utils::get_message_text(&update.message) else {
            return; // ignore messages without text
        };
        let Some(parsed_command) = ParsedCommand::parse(text) else {
            return; // ignore messages without commands
        };
        if let Some(bot_username) = &parsed_command.bot_username {
            if Some(bot_username.to_ascii_lowercase())
                != self
                    .get_me()
                    .unwrap()
                    .username
                    .as_ref()
                    .map(|username| username.to_ascii_lowercase())
            {
                return; // ignore commands sent to other bots
            }
        }
        let Some(command) = self.command_manager.get_command(&parsed_command.name) else {
            return; // ignore nonexistent commands
        };

        self.run_task(command_dispatcher::dispatch_command(
            command,
            parsed_command.arguments,
            CommandContext {
                chat,
                user,
                message: update.message,
                client_id: self.client_id,
                rate_limits: self.rate_limits.clone(),
                message_queue: self.message_queue.clone(),
                http_client: self.http_client.clone(),
            },
        ));
    }

    fn on_new_inline_query(&mut self, update: UpdateNewInlineQuery) {
        self.run_task(calculate_inline::execute(update, self.http_client.clone(), self.client_id));
    }

    fn on_chat_member(&self, update: UpdateChatMember) {
        if let MessageSender::User(user) = &update.new_chat_member.member_id {
            if user.user_id == self.my_id.unwrap() {
                if let Some(chat) = self.cache.get_chat(update.chat_id) {
                    telegram_utils::log_status_update(update, &chat);
                };
            }
        }
    }

    fn on_message_send_succeeded(&mut self, update: UpdateMessageSendSucceeded) {
        self.message_queue.message_sent(Ok(update));
    }

    fn on_message_send_failed(&mut self, update: UpdateMessageSendFailed) {
        self.message_queue.message_sent(Err(update));
    }

    fn on_new_chat(&mut self, update: UpdateNewChat) {
        self.cache.update_new_chat(update);
    }

    fn on_chat_title(&mut self, update: UpdateChatTitle) {
        self.cache.update_chat_title(update);
    }

    fn on_chat_permissions(&mut self, update: UpdateChatPermissions) {
        self.cache.update_chat_permissions(update);
    }

    fn on_user(&mut self, update: UpdateUser) {
        if update.user.id == self.my_id.unwrap() {
            let user = CompactUser::from(update.user.clone());
            log::info!("running as {user}");
        }

        self.cache.update_user(update);
    }

    fn on_option(&mut self, update: UpdateOption) {
        match update.name.as_ref() {
            "my_id" => {
                if let OptionValue::Integer(OptionValueInteger { value }) = update.value {
                    self.my_id = Some(value);
                }
            }
            "version" => {
                if let OptionValue::String(OptionValueString { value }) = update.value {
                    log::info!("running on TDLib {value}");
                }
            }
            _ => (),
        }
    }

    fn on_connection_state(&mut self, update: &UpdateConnectionState) {
        log::info!("connection: {:?}", update.state);

        if let ConnectionState::Ready = update.state {
            self.on_ready();
        }
    }

    pub fn get_me(&self) -> Option<CompactUser> {
        self.cache.get_user(self.my_id?)
    }

    pub fn add_command(&mut self, command: impl CommandTrait + Send + Sync + 'static) {
        self.command_manager.add_command(command);
    }

    pub async fn sync_commands(commands: Vec<BotCommand>, client_id: i32) -> TdResult<()> {
        let BotCommands::BotCommands(bot_commands) =
            functions::get_commands(None, String::new(), client_id).await?;

        if commands == bot_commands.commands {
            log::info!("commands already synced");
            return Ok(());
        }

        let commands_len = commands.len();
        functions::set_commands(None, String::new(), commands, client_id).await?;
        log::info!("synced {commands_len} commands");

        Ok(())
    }
}
