use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::{redirect, Client};
use tgbotapi::requests::{GetMe, GetUpdates};
use tgbotapi::{InlineQuery, Message, Telegram, User};
use tokio::task::JoinHandle;

use crate::not_commands;
use crate::ratelimit::RateLimiter;
use crate::utils::{log_status_update, CommandRef, Context, DisplayUser, ParsedCommand};

pub struct Bot {
    api: Arc<Telegram>,
    running: Arc<AtomicBool>,
    http_client: reqwest::Client,
    me: User,
    commands: HashMap<String, CommandRef>,
    tasks: Vec<JoinHandle<()>>,
    ratelimiter: Arc<RwLock<RateLimiter<(i64, String)>>>,
}

impl Bot {
    pub async fn new() -> Self {
        let api = Arc::new(Telegram::new(env::var("TELEGRAM_TOKEN").unwrap()));
        let me = api.make_request(&GetMe).await.unwrap();
        log::info!("Logged in as {}", me.format_name());
        Self {
            api,
            running: Arc::new(AtomicBool::new(false)),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            me,
            commands: [].into(),
            tasks: Vec::new(),
            ratelimiter: Arc::new(RwLock::new(RateLimiter::new(4, 20))),
        }
    }

    pub async fn run(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        let running_clone = Arc::clone(&self.running);
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            running_clone.store(false, Ordering::Relaxed);
            log::warn!("Stopping…");
        });

        let mut offset = 0;
        loop {
            let updates = match self
                .api
                .make_request(&GetUpdates {
                    offset: Some(offset + 1),
                    timeout: Some(120),
                    allowed_updates: Some(vec![
                        "message".to_string(),
                        "inline_query".to_string(),
                        "my_chat_member".to_string(),
                    ]),
                    ..Default::default()
                })
                .await
            {
                Ok(updates) => updates,
                Err(err) => {
                    log::error!("Error while fetching updates: {err}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            self.tasks.retain(|t| !t.is_finished());

            if !self.running.load(Ordering::Relaxed) {
                break;
            }

            for update in updates {
                if let Some(message) = update.message {
                    self.on_message(message);
                } else if let Some(inline_query) = update.inline_query {
                    self.on_inline_query(inline_query);
                } else if let Some(my_chat_member) = update.my_chat_member {
                    log_status_update(my_chat_member);
                }

                offset = update.update_id;
            }
        }

        if !self.tasks.is_empty() {
            log::info!("Waiting for {} task(s) to finish…", self.tasks.len());
            for task in self.tasks.drain(..) {
                task.await.ok();
            }
        }
    }

    fn spawn_task<T>(&mut self, future: T)
    where
        T: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push(tokio::spawn(future));
    }

    fn on_message(&mut self, message: Message) {
        let user = match message.from.clone() {
            Some(user) => user,
            None => return,
        };

        if user.is_bot || message.forward_from.is_some() {
            return;
        }

        let context = Arc::new(self.get_message_context(message, user));

        if let Some(parsed_command) = ParsedCommand::parse(&context.message) {
            if let Some(command) = self.get_command(&parsed_command) {
                self.dispatch_command(context.clone(), parsed_command, command);
                return;
            }
        };

        self.spawn_task(not_commands::auto_reply(context.clone()));
        self.spawn_task(not_commands::rabbit_nie_je(context));
    }

    fn on_inline_query(&mut self, inline_query: InlineQuery) {
        self.spawn_task(not_commands::calculate_inline(
            self.api.clone(),
            self.http_client.clone(),
            inline_query,
        ));
    }

    fn get_message_context(&self, message: Message, user: User) -> Context {
        Context {
            api: self.api.clone(),
            message,
            user,
            http_client: self.http_client.clone(),
            ratelimiter: self.ratelimiter.clone(),
        }
    }

    fn get_command(&self, parsed_command: &ParsedCommand) -> Option<CommandRef> {
        if let Some(bot_username) = &parsed_command.bot_username {
            if Some(bot_username.to_lowercase())
                != self.me.username.as_ref().map(|u| u.to_lowercase())
            {
                return None;
            }
        }

        self.commands.get(&parsed_command.normalised_name()).map(Arc::clone)
    }

    fn dispatch_command(
        &mut self,
        context: Arc<Context>,
        parsed_command: ParsedCommand,
        command: CommandRef,
    ) {
        let normalised_name = parsed_command.normalised_name();

        if let Some(cooldown) = self
            .ratelimiter
            .write()
            .unwrap()
            .update_rate_limit((context.user.id, normalised_name.clone()), context.message.date)
        {
            log::warn!("Rate limit exceeded by {cooldown}s by {}", context.user.format_name());
            return;
        }

        log::info!("Running {} for {}", parsed_command, context.user.format_name());

        let arguments = parsed_command
            .arguments
            .or_else(|| context.message.reply_to_message.as_ref().and_then(|r| r.text.clone()));

        self.spawn_task(async move {
            if let Err(err) = command.execute(context.clone(), arguments).await {
                log::error!(
                    "An error occurred while executing the {:?} command: {err}",
                    normalised_name
                );
                context.reply("An error occurred while executing the command 😩").await.ok();
            }
        });
    }

    pub fn add_command(&mut self, command: CommandRef) {
        self.commands.insert(command.name().to_string(), command);
    }
}
