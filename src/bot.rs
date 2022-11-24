use std::env;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::{redirect, Client};
use tdlib::enums::{
    self, AuthorizationState, MessageContent, MessageSender, OptionValue, Update, UserType,
};
use tdlib::functions;
use tdlib::types::{
    AuthorizationStateWaitEncryptionKey, Message, MessageText, OptionValueString, TdlibParameters,
    UpdateAuthorizationState, UpdateChatMember, UpdateConnectionState, UpdateNewInlineQuery,
    UpdateNewMessage, UpdateOption, User,
};
use tokio::signal;
use tokio::task::JoinHandle;

use crate::commands::CommandError;
use crate::ratelimit::RateLimiter;
use crate::utils::{
    format_duration, CommandInstance, CommandRef, Context, DisplayUser, ParsedCommand, RateLimits,
};
use crate::{not_commands, utils};

pub struct Bot {
    client_id: i32,
    running: AtomicBool,
    me: Arc<Mutex<Option<User>>>,
    http_client: reqwest::Client,
    commands: Vec<Arc<CommandInstance>>,
    ratelimits: Arc<Mutex<RateLimits>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            client_id: tdlib::create_client(),
            running: AtomicBool::new(false),
            me: Arc::new(Mutex::new(None)),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            commands: Vec::new(),
            ratelimits: Arc::new(Mutex::new(RateLimits {
                ratelimit_exceeded: RateLimiter::new(1, 20),
                auto_reply: RateLimiter::new(2, 20),
            })),
            tasks: Vec::new(),
        }
    }

    pub async fn run(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        let client_id = self.client_id;
        self.run_task(async move {
            functions::set_log_verbosity_level(1, client_id).await.unwrap();
        });

        let client_id = self.client_id;
        self.run_task(async move {
            signal::ctrl_c().await.unwrap();
            functions::close(client_id).await.unwrap();
        });

        while self.running.load(Ordering::Relaxed) {
            self.tasks.retain(|t| !t.is_finished());
            if let Some((update, _)) = tdlib::receive() {
                self.on_update(update);
            }
        }

        if !self.tasks.is_empty() {
            log::info!("waiting for {} task(s) to finish…", self.tasks.len());
            for task in self.tasks.drain(..) {
                task.await.ok();
            }
        }
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
            Update::Option(UpdateOption {
                name,
                value: OptionValue::String(OptionValueString { value }),
            }) => {
                assert!(
                    !(name == "version" && value != "1.8.3"),
                    "unexpected TDLib version {value:?}!"
                );
            }
            Update::ConnectionState(UpdateConnectionState { state }) => {
                println!("connection: {state:?}");
            }
            Update::NewInlineQuery(query) => self.on_inline_query(query),
            Update::ChatMember(update) => self.on_chat_member_update(update),
            _ => (),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn on_authorization_state_update(&mut self, authorization_state: AuthorizationState) {
        {
            println!("authorization: {authorization_state:?}");
            match authorization_state {
                AuthorizationState::WaitTdlibParameters => {
                    let client_id = self.client_id;
                    self.run_task(async move {
                        functions::set_tdlib_parameters(
                            TdlibParameters {
                                database_directory: ".data".into(),
                                api_id: env::var("API_ID").unwrap().parse().unwrap(),
                                api_hash: env::var("API_HASH").unwrap(),
                                system_language_code: "en".into(),
                                device_model: env!("CARGO_PKG_NAME").into(),
                                application_version: env!("CARGO_PKG_VERSION").into(),
                                enable_storage_optimizer: true,
                                ignore_file_names: true,
                                ..Default::default()
                            },
                            client_id,
                        )
                        .await
                        .unwrap();
                    });
                }
                AuthorizationState::WaitEncryptionKey(AuthorizationStateWaitEncryptionKey {
                    ..
                }) => {
                    let client_id = self.client_id;
                    self.run_task(async move {
                        functions::set_database_encryption_key(
                            env::var("DB_ENCRYPTION_KEY").unwrap(),
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
                AuthorizationState::Closed => self.running.store(false, Ordering::Relaxed),
                _ => (),
            }
        }
    }

    fn on_ready(&mut self) {
        let client_id = self.client_id;
        let me = self.me.clone();
        self.run_task(async move {
            let enums::User::User(user) = functions::get_me(client_id).await.unwrap();
            *me.lock().unwrap() = Some(user);
        });
    }

    fn on_message(&mut self, message: Message) {
        let MessageContent::MessageText(MessageText { text, .. }) = &message.content else {
            // ignore non-text messages
            return;
        };

        if message.forward_info.is_some() {
            // ignore forwarded messages
            return;
        }

        let Some(parsed_command) = ParsedCommand::parse(text) else {
            return;
        };

        let Some(command) = self.get_command(&parsed_command) else {
            return;
        };

        self.run_task(Bot::dispatch_command(
            message,
            parsed_command.arguments,
            command,
            self.http_client.clone(),
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

    fn get_command(&self, parsed_command: &ParsedCommand) -> Option<Arc<CommandInstance>> {
        if let Some(bot_username) = &parsed_command.bot_username {
            if Some(bot_username.to_ascii_lowercase())
                != self.me.lock().unwrap().as_ref().map(|me| me.username.to_ascii_lowercase())
            {
                return None;
            }
        }

        self.commands
            .iter()
            .find(|c| {
                c.name == parsed_command.name
                    || c.command_ref.aliases().contains(&parsed_command.name.as_str())
            })
            .cloned()
    }

    #[allow(clippy::too_many_lines)] // TODO: refactor
    async fn dispatch_command(
        message: Message,
        arguments: Option<String>,
        command: Arc<CommandInstance>,
        http_client: reqwest::Client,
        ratelimits: Arc<Mutex<RateLimits>>,
        client_id: i32,
    ) {
        let user = if let MessageSender::User(user) = &message.sender_id {
            let enums::User::User(user) =
                functions::get_user(user.user_id, client_id).await.unwrap();
            user
        } else {
            // ignore messages not sent by users
            return;
        };

        if let UserType::Bot(_) = user.r#type {
            // ignore bots
            return;
        }

        let context = Arc::new(Context { client_id, message, user, http_client, ratelimits });

        let cooldown = command
            .ratelimiter
            .lock()
            .unwrap()
            .update_rate_limit(context.user.id, context.message.date);

        if let Some(cooldown) = cooldown {
            let cooldown_str = format_duration(cooldown.try_into().unwrap());
            log::info!(
                "/{} ratelimit exceeded by {cooldown_str} by {}",
                command.name,
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
            "running /{} {:?} for {} in {}",
            command.name,
            arguments.as_deref().unwrap_or_default(),
            context.user.format_name(),
            chat.title
        );

        if let Err(err) = command.command_ref.execute(context.clone(), arguments).await {
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
                        "Telegram error in the /{} command: {} {}",
                        command.name,
                        err.code,
                        err.message
                    );
                    context
                        .reply(format!("sending the message failed ({}) 😔", err.message))
                        .await
                        .ok();
                }
                CommandError::ReqwestError(err) => {
                    log::error!("HTTP error in the /{} command: {err}", command.name);
                    context.reply(err.without_url().to_string()).await.ok();
                }
            }
        }
    }

    pub fn add_command(&mut self, command: CommandRef) {
        self.commands.push(Arc::new(CommandInstance {
            name: command.name(),
            ratelimiter: Mutex::new(command.rate_limit()),
            command_ref: command,
        }));
    }
}
