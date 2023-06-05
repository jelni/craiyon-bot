use std::sync::Mutex;
use std::time::Duration;

use reqwest::{redirect, Client};

use super::config::Config;
use super::message_queue::MessageQueue;
use super::rate_limit::{RateLimiter, RateLimits};

#[derive(Clone, Copy)]
pub enum BotStatus {
    Running,
    WaitingToClose,
    Closing,
    Closed,
}

pub struct BotState {
    pub client_id: i32,
    pub status: Mutex<BotStatus>,
    pub config: Mutex<Config>,
    pub http_client: Client,
    pub message_queue: MessageQueue,
    pub rate_limits: Mutex<RateLimits>,
}

impl BotState {
    pub fn new(client_id: i32) -> Self {
        Self {
            client_id,
            status: Mutex::new(BotStatus::Closed),
            config: Mutex::new(Config::load().unwrap()),
            http_client: Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            rate_limits: Mutex::new(RateLimits { rate_limit_exceeded: RateLimiter::new(1, 20) }),
            message_queue: MessageQueue::default(),
        }
    }
}
