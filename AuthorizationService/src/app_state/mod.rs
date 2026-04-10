use crate::database_adapter::custom_db_error::BaseDBError;
use crate::database_adapter::{DBAdapter, models};
use crate::jwt_token_manager::JWTTokenManager;
use crate::logger::{LogLevel, Logger};
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    database_adapter: Arc<dyn DBAdapter>,
    token_manager: Arc<JWTTokenManager>,
}

impl AppState {
    pub(crate) fn new(
        logger: Arc<Mutex<dyn Logger>>,
        database_adapter: Arc<dyn DBAdapter>,
    ) -> AppState {
        let jwt_key = env::var("JWT_KEY").unwrap_or(
            "g\"^3'Qd5^LGs,al\\NVNNZ13DU'/+t,0diXj(j9EK0JY2%^'k%.9\\OH2O6p_9Pi])`".to_string(),
        );
        let token_manager = JWTTokenManager::init(jwt_key.as_str());
        match token_manager {
            Err(err) => {
                logger.lock().unwrap().log(
                    format!("Error inializing token manager. Error is: {:?}", err).as_str(),
                    LogLevel::CriticalError,
                );
                panic!()
            }
            Ok(token_manager) => AppState {
                logger: logger.clone(),
                database_adapter: database_adapter.clone(),
                token_manager: Arc::new(token_manager),
            },
        }
    }

    pub(crate) fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }

    pub(crate) async unsafe fn get_all_users_limited(
        &self,
        limit: u64,
    ) -> Result<Vec<models::User>, BaseDBError> {
        Ok(unsafe { self.database_adapter.get_all_user_limited(limit) }.await?)
    }

    pub(crate) async fn create_new_user(
        &self,
        email: String,
        password_hash: String,
    ) -> Result<sqlx::types::Uuid, BaseDBError> {
        Ok(self
            .database_adapter
            .create_new_user(email, password_hash)
            .await?)
    }

    pub(crate) fn get_jwt_token(&self, uuid: sqlx::types::Uuid) -> String {
        self.token_manager.get_jwt_token(uuid.to_string()).unwrap()
    }
}
