use crate::logger::Logger;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct AppState<L: Logger> {
    logger: Arc<L>,
}

impl<L: Logger> AppState<L> {
    pub fn new(logger: L) -> AppState<L> {
        AppState {
            logger: Arc::new(logger)
        }
    }
    
    pub fn get_logger(&self) -> Arc<L> {
        Arc::clone(&self.logger)
    }
}