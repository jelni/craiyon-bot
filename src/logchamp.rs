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

        let (color, light_color) = match record.level() {
            Level::Error => (Color::Red, Color::BrightRed),
            Level::Warn => (Color::Yellow, Color::BrightYellow),
            Level::Info => (Color::Cyan, Color::BrightCyan),
            Level::Debug => (Color::Magenta, Color::BrightMagenta),
            Level::Trace => (Color::Green, Color::BrightGreen),
        };

        let target = record.target().to_string();
        let level = record.level().as_str();
        let len: i32 = (target.len() + level.len()).try_into().unwrap();
        let padding = " ".repeat((48 - len).max(0).try_into().unwrap());

        println!(
            "{} {padding}{} {} {}",
            timestamp.color(Color::BrightBlack),
            target.color(color),
            level.color(light_color),
            record.args()
        );

        writeln!(self.file.lock().unwrap(), "{timestamp} [{target} {level}] {}", record.args())
            .unwrap();
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_boxed_logger(Box::new(Logger::new(".log"))).unwrap();
}
