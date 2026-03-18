use crate::database_adapter::DBAdapter;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Clone)]
pub(crate) struct PostgresDBAdapter {
    pool: Pool<Postgres>,
}

impl DBAdapter for PostgresDBAdapter {
    async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .connect(url)
            .await?;
        Ok(PostgresDBAdapter { pool })
    }
}
