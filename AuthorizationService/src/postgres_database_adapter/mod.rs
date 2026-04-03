use crate::database_adapter::{DBAdapter, models};
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Clone)]
pub(crate) struct PostgresDBAdapter {
    pool: Pool<Postgres>,
}

impl DBAdapter for PostgresDBAdapter {
    async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new().connect(url).await?;
        Ok(PostgresDBAdapter { pool })
    }

    async unsafe fn test_get_all_users(
        &self,
        limit: u64,
    ) -> Result<Vec<models::User>, sqlx::Error> {
        Ok(
            sqlx::query_as::<Postgres, models::User>("SELECT * FROM users LIMIT $1")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await?,
        )
    }
}
