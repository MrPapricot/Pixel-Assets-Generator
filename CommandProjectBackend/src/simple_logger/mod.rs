use crate::logger::{Logger, LogLevel};
use chrono;

#[derive(Copy, Clone)]
pub struct SimpleLogger {}

impl Logger for SimpleLogger {
    fn log(&self, message: &str, log_level: LogLevel) {
        use LogLevel as LL;
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        match log_level {
            LL::Info => println!("Info: {}; Time: {}", message, current_time),
            LL::Warning => println!("Warning: {}; Time: {}", message, current_time),
            LL::Error => eprintln!("Error: {}; Time: {}", message, current_time),
            LL::CriticalError => eprintln!("Critical Error: {}; Time: {}", message, current_time),
        }
    }
}

impl SimpleLogger {
    pub fn new() -> SimpleLogger {
        SimpleLogger {}
    }
}