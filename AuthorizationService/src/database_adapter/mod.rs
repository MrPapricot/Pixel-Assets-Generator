use crate::database_adapter::custom_db_error::BaseDBError;
use sqlx;
use std::pin::Pin;

pub(crate) mod models {
    #[derive(sqlx::FromRow, Debug)]
    pub(crate) struct User {
        uuid: sqlx::types::Uuid,
        pub(crate) email: String,
        password_hash: String,
        pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    }
}

pub(crate) mod custom_db_error {
    #[derive(Debug)]
    pub(crate) enum BaseDBError {
        UniqueViolation,
        BaseError(sqlx::Error),
        RowNotFound,
    }
}

pub(crate) trait DBAdapter: Sync + Send {
    unsafe fn get_all_user_limited<'a>(
        &'a self,
        limit: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<models::User>, BaseDBError>> + Send + 'a>>;

    fn create_new_user<'a>(
        &'a self,
        email: String,
        password: String,
    ) -> Pin<Box<dyn Future<Output = Result<sqlx::types::Uuid, BaseDBError>> + Send + 'a>>;

    fn get_user_by_uuid<'a>(
        &'a self,
        uuid: sqlx::types::Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<models::User, BaseDBError>> + Send + 'a>>;
}
