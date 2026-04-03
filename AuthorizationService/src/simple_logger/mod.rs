use crate::logger::{LogLevel, Logger};
use chrono;

mod colors {
    pub(super) static RESET: &str = "\x1b[0m";
    pub(super) static GREEN: &str = "\x1b[32m";
    pub(super) static YELLOW: &str = "\x1b[33m";
    pub(super) static RED: &str = "\x1b[31m";

    pub(super) static CYAN: &str = "\x1b[36m";
}

pub struct SimpleLogger {}

impl Logger for SimpleLogger {
    fn log(&self, message: &str, log_level: LogLevel) {
        use LogLevel as LL;
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        match log_level {
            LL::Info => println!(
                "{color}Info{reset}: {}; Time: {}",
                message,
                current_time,
                color = colors::GREEN,
                reset = colors::RESET
            ),
            LL::Warning => println!(
                "{color}Warning{reset}: {}; Time: {}",
                message,
                current_time,
                color = colors::CYAN,
                reset = colors::RESET
            ),
            LL::Error => eprintln!(
                "{color}Error{reset}: {}; Time: {}",
                message,
                current_time,
                color = colors::YELLOW,
                reset = colors::RESET
            ),
            LL::CriticalError => eprintln!(
                "{color}Critical Error{reset}: {}; Time: {}",
                message,
                current_time,
                color = colors::RED,
                reset = colors::RESET
            ),
        }
    }
}

impl SimpleLogger {
    pub fn new() -> SimpleLogger {
        SimpleLogger {}
    }
}
