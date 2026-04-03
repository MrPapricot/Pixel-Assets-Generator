use sqlx;

pub(crate) mod models {
    #[derive(sqlx::FromRow, Debug)]
    pub(crate) struct User {
        uuid: sqlx::types::Uuid,
        email: String,
        password_hash: String,
        token: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }
}

pub(crate) trait DBAdapter: Sized + Clone + Sync + Send {
    async fn connect(url: &str) -> Result<Self, sqlx::Error>;

    async unsafe fn test_get_all_users(&self, limit: u64)
    -> Result<Vec<models::User>, sqlx::Error>;
}
