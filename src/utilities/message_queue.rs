use std::collections::HashMap;
use std::sync::Mutex;

use tdlib::types::{Message, UpdateMessageSendFailed, UpdateMessageSendSucceeded};

use crate::bot::{TdError, TdResult};

#[derive(Default)]
pub struct MessageQueue {
    queue: Mutex<HashMap<i64, oneshot::Sender<TdResult<Message>>>>,
}

impl MessageQueue {
    pub async fn wait_for_message(&self, message_id: i64) -> TdResult<Message> {
        self.wait_for_messages(&[message_id]).await.into_iter().next().unwrap()
    }

    pub async fn wait_for_messages(&self, message_ids: &[i64]) -> Vec<TdResult<Message>> {
        let receivers = {
            let mut queue = self.queue.lock().unwrap();
            message_ids
                .iter()
                .map(|&message_id| {
                    let (tx, rx) = oneshot::channel();
                    queue.insert(message_id, tx);
                    rx
                })
                .collect::<Vec<_>>()
        };

        let mut messages = Vec::with_capacity(receivers.len());
        for rx in receivers {
            messages.push(rx.await.unwrap());
        }

        messages
    }

    pub fn message_sent(
        &self,
        result: Result<UpdateMessageSendSucceeded, UpdateMessageSendFailed>,
    ) {
        let (old_message_id, result) = match result {
            Ok(update) => (update.old_message_id, Ok(update.message)),
            Err(update) => (
                update.old_message_id,
                Err(TdError { code: update.error.code, message: update.error.message }),
            ),
        };

        let tx = self.queue.lock().unwrap().remove(&old_message_id);

        if let Some(tx) = tx {
            tx.send(result).unwrap();
        }
    }
}
