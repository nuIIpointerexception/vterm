use log::{error, Level, LevelFilter, Metadata, Record};
use std::panic::PanicInfo;
use std::sync::OnceLock;
use std::time::Instant;

struct Logger {
    time_start: Instant,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with(env!("CARGO_PKG_NAME"))
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let time_now = Instant::now();
            let time = (time_now - self.time_start).as_secs_f64();
            let level = match record.level() {
                Level::Error => "\x1B[1;31mERRO\x1B[0m",
                Level::Warn => "\x1B[1;33mWARN\x1B[0m",
                Level::Info => "\x1B[1;32mINFO\x1B[0m",
                Level::Debug => "\x1B[1;36mDEBG\x1B[0m",
                Level::Trace => "\x1B[1;34mTRCE\x1B[0m",
            };
            println!("[{time:>12.6}] {level} {}", record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn initialize_logger() {
    let time_start = Instant::now();
    let logger = LOGGER.get_or_init(|| Logger { time_start });
    log::set_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace);
}

pub fn initialize_panic_hook() {
    std::panic::set_hook(Box::new(panic_hook));
}

fn panic_hook(info: &PanicInfo) {
    let full_message = info.to_string();
    let message = if let Some((_, message)) = full_message.split_once('\n') {
        message
    } else {
        "panic"
    };
    if let Some(location) = info.location() {
        error!("{message}, \x1B[1mlocation:\x1B[0m {location}");
    } else {
        error!("{message}");
    }
}
