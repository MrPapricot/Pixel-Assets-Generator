use crate::database_adapter::custom_db_error::BaseDBError;
use crate::database_adapter::{DBAdapter, models};
use crate::jwt_token_manager::{JWTDecodingError, JWTTokenManager};
use argon2;
use argon2::{PasswordHasher, PasswordVerifier};
use logger::{LogLevel, Logger};
use std::env;
use std::sync::{Arc, Mutex};

pub(crate) enum Error {
    DBError(BaseDBError),
    HashingError,
}

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    database_adapter: Arc<dyn DBAdapter>,
    token_manager: Arc<JWTTokenManager>,
    argon_salt: argon2::password_hash::SaltString,
    argon: argon2::Argon2<'static>,
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
                logger.lock().expect("poisoned").log(
                    format!("Error initializing token manager. Error is: {:?}", err).as_str(),
                    LogLevel::CriticalError,
                );
                panic!()
            }
            Ok(token_manager) => AppState {
                logger: logger.clone(),
                database_adapter: database_adapter.clone(),
                token_manager: Arc::new(token_manager),
                argon_salt: argon2::password_hash::SaltString::generate(
                    &mut argon2::password_hash::rand_core::OsRng,
                ),
                argon: argon2::Argon2::default(),
            },
        }
    }

    pub(crate) fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }

    #[allow(dead_code)]
    pub(crate) async unsafe fn get_all_users_limited(
        &self,
        limit: u64,
    ) -> Result<Vec<models::User>, BaseDBError> {
        Ok(unsafe { self.database_adapter.get_all_user_limited(limit) }.await?)
    }

    fn compare_passwords(&self, password: String, password_hash: String) -> Option<bool> {
        let hash = argon2::password_hash::PasswordHash::new(password_hash.as_str()).ok()?;
        match self.argon.verify_password(password.as_bytes(), &hash) {
            Ok(()) => Some(true),
            Err(_) => Some(false),
        }
    }

    pub fn get_password_hash(&self, password: String) -> Option<String> {
        self.argon
            .hash_password(password.as_bytes(), &self.argon_salt)
            .and_then(|hash| Ok(hash.to_string()))
            .ok()
    }

    pub(crate) async fn create_new_user(
        &self,
        email: String,
        password: String,
    ) -> Result<String, Error> {
        match self.get_password_hash(password) {
            Some(hash) => match self.database_adapter.create_new_user(email, hash).await {
                Ok(uuid) => Ok(self.get_jwt_token(uuid)),
                Err(db_err) => Err(Error::DBError(db_err)),
            },
            None => Err(Error::HashingError),
        }
    }

    pub(crate) async fn get_user_by_uuid(
        &self,
        uuid: sqlx::types::Uuid,
    ) -> Result<models::User, BaseDBError> {
        Ok(self.database_adapter.get_user_by_uuid(uuid).await?)
    }

    pub(crate) fn get_jwt_token(&self, uuid: sqlx::types::Uuid) -> String {
        self.token_manager.get_jwt_token(uuid)
    }

    pub(crate) fn get_uuid_from_token(
        &self,
        token: &[u8],
    ) -> Result<sqlx::types::Uuid, JWTDecodingError> {
        self.token_manager.get_uuid_from_token(token)
    }

    pub(crate) async fn check_database_health(&self) -> bool {
        self.database_adapter.is_healthy().await
    }

    pub(crate) async fn get_user_uuid(
        &self,
        email: String,
        password: String,
    ) -> Result<String, Error> {
        match self.database_adapter.get_user_by_email(email).await {
            Ok((uuid, password_hash)) => match self.compare_passwords(password, password_hash) {
                Some(equal) => {
                    if equal {
                        Ok(self.get_jwt_token(uuid))
                    } else {
                        Err(Error::DBError(BaseDBError::WrongPassword))
                    }
                }
                None => Err(Error::HashingError),
            },
            Err(error) => Err(Error::DBError(error)),
        }
    }
}
