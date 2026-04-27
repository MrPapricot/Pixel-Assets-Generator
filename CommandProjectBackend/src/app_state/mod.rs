use logger::{LogLevel, Logger};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
}

impl AppState {
    pub fn new(logger: Arc<Mutex<dyn Logger>>) -> AppState {
        AppState {
            logger: logger.clone(),
        }
    }

    pub fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }
}
