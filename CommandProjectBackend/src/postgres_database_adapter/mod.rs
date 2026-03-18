use crate::database_adapter::DBAdapter;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Clone)]
pub(crate) struct PostgresDBAdapter {
    pool: Pool<Postgres>,
}

impl DBAdapter for PostgresDBAdapter {
    async fn connect(
        url: &str,
        min_connections: u32,
        max_connections: u32,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .min_connections(min_connections)
            .max_connections(max_connections)
            .connect(url)
            .await?;
        Ok(PostgresDBAdapter { pool })
    }
}
