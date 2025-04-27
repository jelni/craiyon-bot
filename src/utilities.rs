use std::env;
use log::warn;

pub fn check_env_vars() {
    let example_content = include_str!("../.env.example");
    let mut missing_or_dummy = Vec::new();
    for line in example_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((k, v_example)) = line.split_once('=') {
            let k = k.trim();
            let v_example = v_example.trim_matches('"');
            match env::var(k) {
                Ok(val) if val != v_example => {},
                _ => missing_or_dummy.push((k, v_example)),
            }
        }
    }
    if !missing_or_dummy.is_empty() {
        warn!("Some required environment variables are missing or set to dummy values: {}",
            missing_or_dummy.iter().map(|(k, _)| format!("{k}")).collect::<Vec<_>>().join(", ")
        );
    }
}

pub mod api_utils;
pub mod bot_state;
pub mod cache;
pub mod command_context;
pub mod command_dispatcher;
pub mod command_manager;
pub mod config;
pub mod convert_argument;
pub mod ffprobe;
pub mod file_download;
pub mod google_translate;
pub mod image_utils;
pub mod logchamp;
pub mod markov_chain_manager;
pub mod message_entities;
pub mod message_filters;
pub mod message_queue;
pub mod parsed_command;
pub mod rate_limit;
pub mod telegram_utils;
pub mod text_utils;
pub mod together;
pub mod yt_dlp;

#[cfg(test)]
pub mod test_fixtures;
