use crate::database_adapter::DBAdapter;
use crate::logger::{LogLevel, Logger};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct AppState<DB: DBAdapter> {
    logger: Arc<Mutex<Box<dyn Logger>>>,
    database_adapter: Arc<DB>,
}

impl<DB: DBAdapter> AppState<DB> {
    pub fn new(logger: Box<dyn Logger>, database_adapter: DB) -> AppState<DB> {
        AppState {
            logger: Arc::new(Mutex::new(logger)),
            database_adapter: Arc::new(database_adapter),
        }
    }

    pub fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }
}
