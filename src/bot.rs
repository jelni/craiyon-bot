use std::env;
use std::future::Future;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use reqwest::{redirect, Client};
use tgbotapi::requests::{GetMe, GetUpdates};
use tgbotapi::{InlineQuery, Message, Telegram, Update, User};
use tokio::signal;
use tokio::task::JoinHandle;

use crate::not_commands;
use crate::ratelimit::RateLimiter;
use crate::utils::{
    format_duration, log_status_update, CommandInstance, CommandRef, Context, DisplayUser,
    ParsedCommand, RateLimits,
};

pub struct Bot {
    api: Arc<Telegram>,
    http_client: reqwest::Client,
    me: User,
    commands: Vec<Arc<CommandInstance>>,
    ratelimits: Arc<RwLock<RateLimits>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub async fn new() -> Self {
        let api = Arc::new(Telegram::new(env::var("TELEGRAM_TOKEN").unwrap()));
        let me = api.make_request(&GetMe).await.unwrap();
        log::info!("Logged in as {}", me.format_name());
        Self {
            api,
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            me,
            commands: Vec::new(),
            ratelimits: Arc::new(RwLock::new(RateLimits {
                ratelimit_exceeded: RateLimiter::new(1, 20),
                auto_reply: RateLimiter::new(2, 20),
            })),
            tasks: Vec::new(),
        }
    }

    pub async fn run(&mut self) {
        let mut offset = None;
        loop {
            let updates = tokio::select! {
                biased;
                updates = self.get_updates(offset) => updates,
                _ = signal::ctrl_c() => break,
            };

            let updates = match updates {
                Ok(updates) => updates,
                Err(err) => {
                    log::error!("Error while fetching updates: {err}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            self.tasks.retain(|t| !t.is_finished());

            for update in updates {
                if let Some(message) = update.message {
                    self.on_message(message);
                } else if let Some(inline_query) = update.inline_query {
                    self.on_inline_query(inline_query);
                } else if let Some(my_chat_member) = update.my_chat_member {
                    log_status_update(my_chat_member);
                }

                offset = Some(update.update_id + 1);
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

    async fn get_updates(&self, offset: Option<i32>) -> Result<Vec<Update>, tgbotapi::Error> {
        self.api
            .make_request(&GetUpdates {
                offset,
                timeout: Some(120),
                allowed_updates: Some(vec![
                    "message".to_string(),
                    "inline_query".to_string(),
                    "my_chat_member".to_string(),
                ]),
                ..Default::default()
            })
            .await
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

        let parsed_command = ParsedCommand::parse(&context.message);
        let command = parsed_command.as_ref().and_then(|command| self.get_command(command));

        self.spawn_task(async move {
            if context.message.forward_from_chat.is_some() {
                not_commands::rabbit_nie_je(context).await;
                return;
            }

            if let Some(command) = command {
                Bot::dispatch_command(context.clone(), parsed_command.unwrap(), command).await;
                return;
            };

            not_commands::auto_reply(context).await;
        });
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
            ratelimits: self.ratelimits.clone(),
        }
    }

    fn get_command(&self, parsed_command: &ParsedCommand) -> Option<Arc<CommandInstance>> {
        if let Some(bot_username) = &parsed_command.bot_username {
            if Some(bot_username.to_ascii_lowercase())
                != self.me.username.as_ref().map(|u| u.to_ascii_lowercase())
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

    async fn dispatch_command(
        context: Arc<Context>,
        parsed_command: ParsedCommand,
        command: Arc<CommandInstance>,
    ) {
        let cooldown = command
            .ratelimiter
            .write()
            .unwrap()
            .update_rate_limit(context.user.id, context.message.date);

        if let Some(cooldown) = cooldown {
            let cooldown_str = format_duration(cooldown.try_into().unwrap());
            log::warn!(
                "/{} ratelimit exceeded by {cooldown_str} by {}",
                command.name,
                context.user.format_name()
            );
            if context
                .ratelimits
                .write()
                .unwrap()
                .ratelimit_exceeded
                .update_rate_limit(context.user.id, context.message.date)
                .is_none()
            {
                let cooldown_end =
                    Instant::now() + Duration::from_secs(cooldown.max(5).try_into().unwrap());
                if let Ok(message) = context
                    .reply(format!("You can use this command again in {cooldown_str}."))
                    .await
                {
                    tokio::time::sleep_until(cooldown_end.into()).await;
                    context.delete_message(&message).await.ok();
                }
            }
            return;
        }

        log::info!(
            "Running /{} {:?} for {}",
            command.name,
            parsed_command.arguments.as_deref().unwrap_or_default(),
            context.user.format_name()
        );

        let arguments = parsed_command
            .arguments
            .or_else(|| context.message.reply_to_message.as_ref().and_then(|r| r.text.clone()));

        if let Err(err) = command.command_ref.execute(context.clone(), arguments).await {
            log::error!(
                "An error occurred while executing the {:?} command: {err}",
                parsed_command.name
            );
            context.reply("An error occurred while executing the command 😩").await.ok();
        }
    }

    pub fn add_command(&mut self, command: CommandRef) {
        self.commands.push(Arc::new(CommandInstance {
            name: command.name(),
            ratelimiter: RwLock::new(command.rate_limit()),
            command_ref: command,
        }));
    }
}
