use std::env;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use markov_chain::MarkovChain;
use reqwest::{Client, redirect};
use tdlib::enums::{ChatMember, ChatMemberStatus, MessageSender};
use tdlib::functions;
use tdlib::types::MessageSenderUser;
use tokio::sync::Mutex as TokioMutex;

use super::cache::Cache;
use super::config::Config;
use super::markov_chain_manager;
use super::message_queue::MessageQueue;
use super::rate_limit::{RateLimiter, RateLimits};
use crate::apis::coinranking::Coin;
use crate::apis::eurofxref::Rate;
use crate::bot::TdResult;

#[derive(Clone, Copy)]
pub enum BotStatus {
    Running,
    WaitingToClose,
    Closing,
    Closed,
}

pub struct Currencies {
    pub updated_at: Instant,
    pub fiat: Vec<Rate>,
    pub crypto: Vec<Coin>,
}

pub struct BotState {
    pub status: Mutex<BotStatus>,
    pub config: Mutex<Config>,
    pub cache: Mutex<Cache>,
    pub http_client: Client,
    pub currencies: TokioMutex<Option<Currencies>>,
    pub message_queue: MessageQueue,
    pub rate_limits: Mutex<RateLimits>,
    pub markov_chain: Mutex<MarkovChain>,
}

impl BotState {
    pub fn new() -> Self {
        let mut http_client = Client::builder();

        if let Ok(user_agent) = env::var("USER_AGENT") {
            http_client = http_client.user_agent(user_agent);
        }

        Self {
            status: Mutex::new(BotStatus::Closed),
            config: Mutex::new(Config::load().unwrap()),
            cache: Mutex::new(Cache::default()),
            http_client: http_client
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_mins(5))
                .build()
                .unwrap(),
            currencies: TokioMutex::new(None),
            rate_limits: Mutex::new(RateLimits { rate_limit_exceeded: RateLimiter::new(1, 20) }),
            message_queue: MessageQueue::default(),
            markov_chain: Mutex::new(markov_chain_manager::load().unwrap()),
        }
    }

    pub async fn get_member_status(
        &self,
        chat_id: i64,
        member_id: i64,
        client_id: i32,
    ) -> TdResult<ChatMemberStatus> {
        let status = self.cache.lock().unwrap().get_member_status(chat_id, member_id);

        if let Some(status) = status {
            return Ok(status);
        }

        let ChatMember::ChatMember(chat_member) = functions::get_chat_member(
            chat_id,
            MessageSender::User(MessageSenderUser { user_id: member_id }),
            client_id,
        )
        .await?;

        self.cache.lock().unwrap().set_member_status(
            chat_id,
            member_id,
            chat_member.status.clone(),
        );

        Ok(chat_member.status)
    }
}
