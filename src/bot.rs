use std::env;
use std::future::Future;
use std::sync::Arc;

use tdlib::enums::{
    AuthorizationState, BotCommands, ConnectionState, MessageSender, OptionValue, Update,
};
use tdlib::functions;
use tdlib::types::{
    BotCommand, OptionValueInteger, OptionValueString, UpdateAuthorizationState, UpdateChatMember,
    UpdateChatPermissions, UpdateChatTitle, UpdateConnectionState, UpdateMessageSendFailed,
    UpdateMessageSendSucceeded, UpdateNewChat, UpdateNewInlineQuery, UpdateNewMessage,
    UpdateOption, UpdateUser,
};
use tokio::signal;
use tokio::task::JoinHandle;

use crate::commands::{calculate_inline, dice_reply, CommandTrait};
use crate::utilities::bot_state::{BotState, BotStatus};
use crate::utilities::cache::CompactUser;
use crate::utilities::command_manager::{CommandInstance, CommandManager};
use crate::utilities::message_filters::MessageDestination;
use crate::utilities::{command_dispatcher, markov_chain_manager, message_filters, telegram_utils};

pub type TdError = tdlib::types::Error;
pub type TdResult<T> = Result<T, TdError>;

pub struct Bot {
    pub client_id: i32,
    my_id: Option<i64>,
    command_manager: CommandManager,
    state: Arc<BotState>,
    tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            client_id: tdlib::create_client(),
            my_id: None,
            command_manager: CommandManager::new(),
            state: Arc::new(BotState::new()),
            tasks: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        *self.state.status.lock().unwrap() = BotStatus::Running;
        let client_id = self.client_id;
        self.run_task(async move {
            functions::set_log_verbosity_level(1, client_id).await.unwrap();
        });

        let state = self.state.clone();
        self.run_task(async move {
            signal::ctrl_c().await.unwrap();
            log::warn!("Ctrl+C received");
            *state.status.lock().unwrap() = BotStatus::WaitingToClose;
        });

        let mut last_task_count = 0;
        loop {
            if let Some((update, _)) = tdlib::receive() {
                self.on_update(update);
            }
            self.tasks.retain(|t| !t.is_finished());
            let state = *self.state.status.lock().unwrap();
            match state {
                BotStatus::WaitingToClose => {
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
                BotStatus::Closed => break,
                _ => (),
            }
        }

        if let Err(err) = self.state.config.lock().unwrap().save() {
            log::error!("failed to save bot config: {err}");
        }

        if let Err(err) = markov_chain_manager::save(&self.state.markov_chain.lock().unwrap()) {
            log::error!("failed to save Markov chain: {err}");
        }
    }

    fn close(&mut self) {
        *self.state.status.lock().unwrap() = BotStatus::Closing;
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
            Update::AuthorizationState(update) => self.on_authorization_state(&update),
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

    fn on_authorization_state(&mut self, update: &UpdateAuthorizationState) {
        log::info!("authorization: {:?}", update.authorization_state);

        match update.authorization_state {
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
            AuthorizationState::Closed => *self.state.status.lock().unwrap() = BotStatus::Closed,
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
        if let Some(destination) =
            message_filters::filter_message(self, self.state.clone(), update.message)
        {
            match destination {
                MessageDestination::Command { command, arguments, context } => {
                    self.run_task(command_dispatcher::dispatch_command(
                        command, arguments, context,
                    ));
                }
                MessageDestination::Dice { message } => {
                    self.run_task(dice_reply::execute(message, self.client_id));
                }
                MessageDestination::MarkovChain { text } => {
                    markov_chain_manager::train(&mut self.state.markov_chain.lock().unwrap(), text);
                }
            }
        }
    }

    fn on_new_inline_query(&mut self, update: UpdateNewInlineQuery) {
        self.run_task(calculate_inline::execute(
            update,
            self.state.http_client.clone(),
            self.client_id,
        ));
    }

    fn on_chat_member(&mut self, update: UpdateChatMember) {
        if let MessageSender::User(user) = &update.new_chat_member.member_id {
            if self.my_id.is_some_and(|my_id| user.user_id == my_id) {
                if let Some(chat) = self.state.cache.lock().unwrap().get_chat(update.chat_id) {
                    telegram_utils::log_status_update(&update, &chat);
                };
            }
        }

        self.state.cache.lock().unwrap().update_chat_member(update);
    }

    fn on_message_send_succeeded(&mut self, update: UpdateMessageSendSucceeded) {
        self.state.message_queue.message_sent(Ok(update));
    }

    fn on_message_send_failed(&mut self, update: UpdateMessageSendFailed) {
        self.state.message_queue.message_sent(Err(update));
    }

    fn on_new_chat(&mut self, update: UpdateNewChat) {
        self.state.cache.lock().unwrap().update_new_chat(update);
    }

    fn on_chat_title(&mut self, update: UpdateChatTitle) {
        self.state.cache.lock().unwrap().update_chat_title(update);
    }

    fn on_chat_permissions(&mut self, update: UpdateChatPermissions) {
        self.state.cache.lock().unwrap().update_chat_permissions(update);
    }

    fn on_user(&mut self, update: UpdateUser) {
        if self.my_id.is_some_and(|my_id| update.user.id == my_id) {
            let user = CompactUser::from(update.user.clone());
            log::info!("running as {user}");
        }

        self.state.cache.lock().unwrap().update_user(update);
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
        self.state.cache.lock().unwrap().get_user(self.my_id?)
    }

    pub fn add_command(&mut self, command: impl CommandTrait + Send + Sync + 'static) {
        self.command_manager.add_command(Box::new(command));
    }

    pub fn get_command(&self, name: &str) -> Option<Arc<CommandInstance>> {
        self.command_manager.get_command(name)
    }

    pub async fn sync_commands(commands: Vec<BotCommand>, client_id: i32) -> TdResult<()> {
        let BotCommands::BotCommands(bot_commands) =
            functions::get_commands(None, String::new(), client_id).await?;

        if commands == bot_commands.commands {
            log::debug!("commands already synced");
            return Ok(());
        }

        let commands_len = commands.len();
        functions::set_commands(None, String::new(), commands, client_id).await?;
        log::info!("synced {commands_len} commands");

        Ok(())
    }
}
