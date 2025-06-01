//! joe: logchamp
//! joe: call it logchamp
//! joe: please

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Mutex;

use colored::{Color, Colorize};
use log::{Level, LevelFilter, Log, Metadata, Record};
use time::macros;

struct Logger {
    file: Mutex<BufWriter<File>>,
}

impl Logger {
    fn new(filename: &str) -> Self {
        log::set_max_level(LevelFilter::Debug);
        Self { file: Mutex::new(BufWriter::new(File::create(filename).unwrap())) }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.target().split("::").next().unwrap() {
            "craiyon_bot" => true,
            _ => metadata.level() <= Level::Info,
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = time::OffsetDateTime::now_utc()
            .format(macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"))
            .unwrap();
        let target = record.target();
        let level = record.level().as_str();
        let args = record.args();

        let color = match record.level() {
            Level::Error => Color::BrightRed,
            Level::Warn => Color::BrightYellow,
            Level::Info => Color::BrightCyan,
            Level::Debug => Color::Magenta,
            Level::Trace => Color::Green,
        };

        println!("{} {} {args}", timestamp.color(Color::BrightBlack), level.color(color));
        writeln!(self.file.lock().unwrap(), "{timestamp} [{target} {level}] {args}").unwrap();
    }

    fn flush(&self) {
        self.file.lock().unwrap().flush().unwrap();
    }
}

pub fn init() {
    log::set_boxed_logger(Box::new(Logger::new(".log"))).unwrap();
}
