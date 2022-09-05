use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::{redirect, Client};
use tgbotapi::requests::{
    AnswerInlineQuery, GetUpdates, InlineQueryResult, InlineQueryResultArticle, InlineQueryType,
    InputMessageText, InputMessageType,
};
use tgbotapi::{InlineQuery, Message, Telegram};
use tokio::task::JoinHandle;

use crate::commands::Command;
use crate::utils::{Context, DisplayUser, ParsedCommand};
use crate::{mathjs, utils};

type CommandRef = Arc<dyn Command + Send + Sync>;

pub struct Bot {
    pub api: Arc<Telegram>,
    pub running: Arc<AtomicBool>,
    pub http_client: reqwest::Client,
    pub commands: HashMap<String, CommandRef>,
    pub tasks: Vec<JoinHandle<()>>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            api: Arc::new(Telegram::new(env::var("TELEGRAM_TOKEN").unwrap())),
            running: Arc::new(AtomicBool::new(false)),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            commands: [].into(),
            tasks: Vec::new(),
        }
    }

    pub async fn run(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        let running_clone = Arc::clone(&self.running);
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            running_clone.store(false, Ordering::Relaxed);
            log::warn!("Stopping…")
        });

        let mut offset = 0;
        loop {
            let updates = match self
                .api
                .make_request(&GetUpdates {
                    offset: Some(offset + 1),
                    timeout: Some(120),
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

            if !self.running.load(Ordering::Relaxed) {
                break;
            }

            for update in updates {
                if let Some(message) = update.message {
                    if let Err(err) = self.on_message(message).await {
                        log::error!("Error in on_message: {err}");
                    }
                } else if let Some(inline_query) = update.inline_query {
                    if let Err(err) = self.on_inline_query(inline_query).await {
                        log::error!("Error in on_inline_query: {err}");
                    }
                }

                offset = update.update_id;
            }
        }

        log::info!("Waiting for tasks to finish…");
        for task in self.tasks.drain(..) {
            task.await.ok();
        }
    }

    async fn on_message(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        if let Some(user) = &message.from {
            if !user.is_bot && message.forward_from.is_none() {
                if let Some(parsed_command) = ParsedCommand::parse(&message) {
                    if let Some(command) = self.commands.get(&parsed_command.normalised_name()) {
                        let arguments = parsed_command
                            .arguments
                            .clone()
                            .or_else(|| message.reply_to_message.clone().and_then(|r| r.text));
                        let context = self.get_message_context(message.clone(), arguments);
                        self.run_command(command.clone(), context, parsed_command);
                    }
                }
            }
        }

        utils::rabbit_nie_je(self.get_message_context(message, None)).await.ok();

        Ok(())
    }

    async fn on_inline_query(&self, inline_query: InlineQuery) -> Result<(), Box<dyn Error>> {
        let query = inline_query.query;
        if query.is_empty() {
            self.api
                .make_request(&AnswerInlineQuery {
                    inline_query_id: inline_query.id,
                    results: Vec::new(),
                    ..Default::default()
                })
                .await?;

            return Ok(());
        }

        let (title, message_text) = if query.split_ascii_whitespace().collect::<String>() == "2+2" {
            ("5".to_string(), format!("{} = 5", query))
        } else {
            match mathjs::evaluate(self.http_client.clone(), query.clone()).await? {
                Ok(result) => (result.clone(), format!("{} = {}", query, result)),
                Err(err) => (err.clone(), err),
            }
        };

        self.api
            .make_request(&AnswerInlineQuery {
                inline_query_id: inline_query.id,
                results: vec![InlineQueryResult {
                    id: "0".to_string(),
                    result_type: "article".to_string(),
                    content: InlineQueryType::Article(InlineQueryResultArticle {
                        title,
                        input_message_content: InputMessageType::Text(InputMessageText {
                            message_text,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    reply_markup: None,
                }],
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    fn get_message_context(&self, message: Message, arguments: Option<String>) -> Context {
        Context { api: self.api.clone(), message, arguments, http_client: self.http_client.clone() }
    }

    fn run_command(
        &mut self,
        command: CommandRef,
        context: Context,
        parsed_command: ParsedCommand,
    ) {
        log::info!(
            "Running {parsed_command} for {}",
            context.message.from.as_ref().unwrap().format_name()
        );
        self.tasks.retain(|t| !t.is_finished());
        self.tasks.push(tokio::spawn(async move {
            if let Err(err) = command.execute(context).await {
                let error_text = format!(
                    "An error occurred while executing the {:?} command: {err}",
                    parsed_command.normalised_name()
                );
                log::error!("{error_text}");
            }
        }));
    }

    pub fn add_command<S: Into<String>>(&mut self, name: S, command: CommandRef) {
        self.commands.insert(name.into(), command);
    }
}
