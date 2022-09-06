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
        match self.history.get_mut(&key) {
            Some(value) => {
                let cooldown = match value.len() >= self.limit {
                    true => {
                        value.truncate(self.limit);
                        let last_time = *value.last().unwrap();
                        match time - last_time < self.duration {
                            true => Some(self.duration - (time - last_time)),
                            false => None,
                        }
                    }
                    false => None,
                };

                if cooldown.is_none() {
                    value.insert(0, time);
                }
                cooldown
            }
            None => {
                self.history.insert(key, vec![time]);
                None
            }
        }
    }
}
