use crate::logger::Logger;
use std::sync::Arc;
use crate::database_adapter::DBAdapter;

#[derive(Clone)]
pub(crate) struct AppState<L: Logger, DB: DBAdapter> {
    logger: Arc<L>,
    db_adapter: Arc<DB>,
}

impl<L: Logger, DB: DBAdapter> AppState<L, DB> {
    pub fn new(logger: L, db_adapter: DB) -> AppState<L, DB> {
        AppState {
            logger: Arc::new(logger),
            db_adapter: Arc::new(db_adapter),
        }
    }

    pub fn get_logger(&self) -> Arc<L> {
        Arc::clone(&self.logger)
    }
}
