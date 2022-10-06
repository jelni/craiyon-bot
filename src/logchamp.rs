//! joe: logchamp
//! joe: call it logchamp
//! joe: please

use colored::{Color, Colorize};
use log::{Level, Log, Metadata, Record};
use time::macros::format_description;

static LOGGER: Logger = Logger;

struct Logger;

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

        let color = match record.level() {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Blue,
            Level::Debug => Color::Green,
            Level::Trace => Color::Cyan,
        };

        let target = record.target().to_string();
        let level = record.level().as_str();
        let len: i32 = (target.len() + level.len()).try_into().unwrap();
        let padding = " ".repeat((16 - len).max(0).try_into().unwrap());

        println!(
            "{} {padding}{} {} {}",
            time::OffsetDateTime::now_utc()
                .format(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"))
                .unwrap()
                .color(Color::BrightBlack),
            target.color(color),
            level.color(color).bold(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
}
