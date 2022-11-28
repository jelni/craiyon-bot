use std::env;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::{redirect, Client};
use tdlib::enums::{
    self, AuthorizationState, BotCommands, MessageContent, MessageSender, Update, UserType,
};
use tdlib::functions;
use tdlib::types::{
    BotCommand, Message, MessageText, UpdateAuthorizationState, UpdateChatMember,
    UpdateConnectionState, UpdateMessageSendFailed, UpdateMessageSendSucceeded,
    UpdateNewInlineQuery, UpdateNewMessage, User,
};
use tokio::signal;
use tokio::task::JoinHandle;

use crate::command_manager::{CommandInstance, CommandManager, CommandRef};
use crate::commands::CommandError;
use crate::message_queue::MessageQueue;
use crate::ratelimit::RateLimiter;
use crate::utils::{format_duration, Context, DisplayUser, ParsedCommand, RateLimits};
use crate::{not_commands, utils};

pub type TdError = tdlib::types::Error;

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
    me: Arc<Mutex<Option<User>>>,
    http_client: reqwest::Client,
    command_manager: CommandManager,
    message_queue: Arc<MessageQueue>,
    ratelimits: Arc<Mutex<RateLimits>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            client_id: tdlib::create_client(),
            state: Arc::new(Mutex::new(BotState::Closed)),
            me: Arc::new(Mutex::new(None)),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            command_manager: CommandManager::new(),
            ratelimits: Arc::new(Mutex::new(RateLimits {
                ratelimit_exceeded: RateLimiter::new(1, 20),
                auto_reply: RateLimiter::new(2, 20),
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
                        log::info!("waiting for {} task(s) to finish…", self.tasks.len());
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
            Update::AuthorizationState(UpdateAuthorizationState { authorization_state }) => {
                self.on_authorization_state_update(authorization_state);
            }
            Update::NewMessage(UpdateNewMessage { message }) => self.on_message(message),
            Update::MessageSendSucceeded(update) => self.on_message_sent(Ok(update)),
            Update::MessageSendFailed(update) => self.on_message_sent(Err(update)),
            Update::ConnectionState(UpdateConnectionState { state }) => {
                log::info!("connection: {state:?}");
            }
            Update::NewInlineQuery(query) => self.on_inline_query(query),
            Update::ChatMember(update) => self.on_chat_member_update(update),
            _ => (),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn on_authorization_state_update(&mut self, authorization_state: AuthorizationState) {
        {
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
                            false,
                            false,
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
                AuthorizationState::Ready => self.on_ready(),
                AuthorizationState::Closed => *self.state.lock().unwrap() = BotState::Closed,
                _ => (),
            }
        }
    }

    fn on_ready(&mut self) {
        let client_id = self.client_id;
        let me = self.me.clone();
        let commands = self.command_manager.public_command_list();
        self.run_task(async move {
            let enums::User::User(user) = functions::get_me(client_id).await.unwrap();
            log::info!(
                "running as @{}",
                user.usernames.as_ref().map(|u| u.editable_username.as_str()).unwrap_or_default()
            );
            *me.lock().unwrap() = Some(user);
            Bot::sync_commands(commands, client_id).await.unwrap();
        });
    }

    fn on_message(&mut self, message: Message) {
        let BotState::Running = *self.state.lock().unwrap() else {
            return; // ignore when closing
        };
        let MessageContent::MessageText(MessageText { text, .. }) = &message.content else {
            return; // ignore non-text messages
        };
        if message.forward_info.is_some() {
            return; // ignore forwarded messages
        }
        let Some(parsed_command) = ParsedCommand::parse(text) else {
            return; // ignore messages without commands
        };
        if let Some(bot_username) = &parsed_command.bot_username {
            if Some(bot_username.to_ascii_lowercase())
                != self.me.lock().unwrap().as_ref().map(|me| {
                    me.usernames
                        .as_ref()
                        .map(|u| u.editable_username.to_ascii_lowercase())
                        .unwrap_or_default()
                })
            {
                return; // ignore commands sent to other bots
            }
        }
        let Some(command) = self.command_manager.get_command(&parsed_command.name) else {
            return; // ignore nonexistent commands
        };

        self.run_task(Bot::dispatch_command(
            message,
            parsed_command.arguments,
            command,
            self.http_client.clone(),
            self.message_queue.clone(),
            self.ratelimits.clone(),
            self.client_id,
        ));
    }

    fn on_inline_query(&mut self, query: UpdateNewInlineQuery) {
        self.run_task(not_commands::calculate_inline(
            query,
            self.http_client.clone(),
            self.client_id,
        ));
    }

    fn on_chat_member_update(&mut self, update: UpdateChatMember) {
        self.run_task(utils::log_status_update(update, self.client_id));
    }

    fn on_message_sent(
        &mut self,
        result: Result<UpdateMessageSendSucceeded, UpdateMessageSendFailed>,
    ) {
        self.message_queue.message_sent(result);
    }

    #[allow(clippy::too_many_lines)] // TODO: refactor
    async fn dispatch_command(
        message: Message,
        arguments: Option<String>,
        command: Arc<CommandInstance>,
        http_client: reqwest::Client,
        message_queue: Arc<MessageQueue>,
        ratelimits: Arc<Mutex<RateLimits>>,
        client_id: i32,
    ) {
        let user = if let MessageSender::User(user) = &message.sender_id {
            let enums::User::User(user) =
                functions::get_user(user.user_id, client_id).await.unwrap();
            user
        } else {
            return; // ignore messages not sent by users
        };

        if let UserType::Bot(_) = user.r#type {
            return; // ignore bots
        }

        let context =
            Arc::new(Context { client_id, message, user, http_client, message_queue, ratelimits });

        let cooldown = command
            .ratelimiter
            .lock()
            .unwrap()
            .update_rate_limit(context.user.id, context.message.date);

        if let Some(cooldown) = cooldown {
            let cooldown_str = format_duration(cooldown.try_into().unwrap());
            log::info!(
                "{command} ratelimit exceeded by {cooldown_str} by {}",
                context.user.format_name()
            );
            if context
                .ratelimits
                .lock()
                .unwrap()
                .ratelimit_exceeded
                .update_rate_limit(context.user.id, context.message.date)
                .is_none()
            {
                let cooldown_end =
                    Instant::now() + Duration::from_secs(cooldown.max(5).try_into().unwrap());
                if let Ok(message) = context
                    .reply(format!("you can use this command again in {cooldown_str}."))
                    .await
                {
                    tokio::time::sleep_until(cooldown_end.into()).await;
                    context.delete_message(message.id).await.ok();
                }
            }
            return;
        }

        let arguments = match arguments {
            None => {
                if context.message.reply_to_message_id == 0 {
                    None
                } else {
                    match functions::get_message(
                        context.message.chat_id,
                        context.message.reply_to_message_id,
                        client_id,
                    )
                    .await
                    {
                        Ok(message) => {
                            let enums::Message::Message(message) = message;
                            if let MessageContent::MessageText(text) = message.content {
                                Some(text.text.text)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                }
            }
            arguments => arguments,
        };

        let chat = match functions::get_chat(context.message.chat_id, client_id).await {
            Ok(chat) => {
                let enums::Chat::Chat(chat) = chat;
                chat
            }
            Err(_) => return,
        };

        log::info!(
            "running {command} {:?} for {} in {}",
            arguments.as_deref().unwrap_or_default(),
            context.user.format_name(),
            chat.title
        );

        if let Err(err) = command.command.execute(context.clone(), arguments).await {
            match err {
                CommandError::CustomError(text) => {
                    context.reply(text).await.ok();
                }
                CommandError::CustomMarkdownError(text) => {
                    context.reply_markdown(text).await.ok();
                }
                CommandError::MissingArgument(argument) => {
                    context.reply(format!("missing {argument}.")).await.ok();
                }
                CommandError::TelegramError(err) => {
                    log::error!(
                        "Telegram error in the {command} command: {} {}",
                        err.code,
                        err.message
                    );
                    context
                        .reply(format!("sending the message failed ({}) 😔", err.message))
                        .await
                        .ok();
                }
                CommandError::ReqwestError(err) => {
                    log::error!("HTTP error in the {command} command: {err}");
                    context.reply(err.without_url().to_string()).await.ok();
                }
            }
        }
    }

    pub fn add_command(&mut self, command: CommandRef) {
        self.command_manager.add_command(command);
    }

    pub async fn sync_commands(commands: Vec<BotCommand>, client_id: i32) -> Result<(), TdError> {
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
