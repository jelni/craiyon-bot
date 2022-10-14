//! joe: logchamp
//! joe: call it logchamp
//! joe: please

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

use colored::{Color, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record};
use time::macros::format_description;

struct Logger {
    file: Mutex<File>,
}

impl Logger {
    fn new(filename: &str) -> Self {
        log::set_max_level(LevelFilter::Debug);
        Self {
            file: Mutex::new(
                OpenOptions::new().write(true).truncate(true).create(true).open(filename).unwrap(),
            ),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let max_level = match metadata.target().split("::").next().unwrap() {
            "craiyon-bot" => Level::Debug,
            _ => Level::Info,
        };
        metadata.level() <= max_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = time::OffsetDateTime::now_utc()
            .format(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"))
            .unwrap();
        let target = record.target().to_string();
        let level = record.level().as_str();
        let args = record.args();

        let color = match record.level() {
            Level::Error => Color::BrightRed,
            Level::Warn => Color::BrightYellow,
            Level::Info => Color::BrightCyan,
            Level::Debug => Color::BrightMagenta,
            Level::Trace => Color::BrightGreen,
        };

        println!("{} {} {args}", timestamp.color(Color::BrightBlack), level.color(color));
        writeln!(self.file.lock().unwrap(), "{timestamp} [{target} {level}] {args}").unwrap();
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_boxed_logger(Box::new(Logger::new(".log"))).unwrap();
}
