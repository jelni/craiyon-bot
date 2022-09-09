use std::collections::HashMap;
use std::hash::Hash;

pub struct RateLimiter<K: Eq + Hash> {
    limit: usize,
    duration: i64,
    history: HashMap<K, Vec<i64>>,
}

impl<K: Eq + Hash> RateLimiter<K> {
    pub fn new(limit: usize, duration: i64) -> Self {
        Self { limit, duration, history: HashMap::new() }
    }

    pub fn update_rate_limit(&mut self, key: K, time: i64) -> Option<i64> {
        if let Some(value) = self.history.get_mut(&key) {
            let cooldown = if value.len() >= self.limit {
                value.truncate(self.limit);
                let last_time = *value.last().unwrap();
                if time - last_time < self.duration {
                    Some(self.duration - (time - last_time))
                } else {
                    None
                }
            } else {
                None
            };

            if cooldown.is_none() {
                value.insert(0, time);
            }
            cooldown
        } else {
            self.history.insert(key, vec![time]);
            None
        }
    }
}
