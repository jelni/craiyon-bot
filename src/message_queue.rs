use std::collections::HashMap;
use std::sync::Mutex;

use tdlib::types::{Message, UpdateMessageSendFailed, UpdateMessageSendSucceeded};

use crate::bot::TdError;

#[derive(Default)]
pub struct MessageQueue {
    queue: Mutex<HashMap<i64, oneshot::Sender<Result<Message, TdError>>>>,
}

impl MessageQueue {
    pub async fn wait_for_message(&self, message_id: i64) -> Result<Message, TdError> {
        let (tx, rx) = oneshot::channel();
        self.queue.lock().unwrap().insert(message_id, tx);
        rx.await.unwrap()
    }

    pub fn message_sent(
        &self,
        result: Result<UpdateMessageSendSucceeded, UpdateMessageSendFailed>,
    ) {
        let (old_message_id, result) = match result {
            Ok(update) => (update.old_message_id, Ok(update.message)),
            Err(update) => (
                update.old_message_id,
                Err(TdError { code: update.error_code, message: update.error_message }),
            ),
        };

        if let Some(tx) = self.queue.lock().unwrap().remove(&old_message_id) {
            tx.send(result).unwrap();
        }
    }
}
