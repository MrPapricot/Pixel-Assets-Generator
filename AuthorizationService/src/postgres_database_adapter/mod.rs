use crate::database_adapter::{DBAdapter, custom_db_error::BaseDBError, models};
use sqlx::error::DatabaseError;
use sqlx::pool::Pool;
use sqlx::postgres::{PgDatabaseError, PgPoolOptions, Postgres};
use std::pin::Pin;

#[derive(Clone)]
pub(crate) struct PostgresDBAdapter {
    pool: Pool<Postgres>,
}

impl PostgresDBAdapter {
    pub(crate) async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .test_before_acquire(true)
            .connect(url)
            .await?;
        Ok(PostgresDBAdapter { pool })
    }
}

impl DBAdapter for PostgresDBAdapter {
    // async unsafe fn test_get_all_users(
    //     &self,
    //     limit: u64,
    // ) -> Result<Vec<models::User>, sqlx::Error> {
    //     todo!();
    // }
    unsafe fn get_all_user_limited<'a>(
        &'a self,
        limit: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<models::User>, BaseDBError>> + Send + 'a>> {
        Box::pin(async move {
            return match sqlx::query_as::<Postgres, models::User>("SELECT * FROM users LIMIT $1")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
            {
                Ok(users) => Ok(users),
                Err(error) => Err(BaseDBError::BaseError(error)),
            };
        })
    }

    fn create_new_user<'a>(
        &'a self,
        email: String,
        password_hash: String,
    ) -> Pin<Box<dyn Future<Output = Result<sqlx::types::Uuid, BaseDBError>> + Send + 'a>> {
        Box::pin(async move {
            #[derive(sqlx::FromRow)]
            struct ReturnType(sqlx::types::Uuid);

            let res = sqlx::query_as::<Postgres, ReturnType>(
                "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING uuid",
            )
            .bind(email)
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await;
            return match res {
                Ok(ReturnType(uuid)) => Ok(uuid),
                Err(sqlx_err) => match sqlx_err {
                    sqlx::Error::Database(db_err) => {
                        if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
                            if pg_err.is_unique_violation() {
                                Err(BaseDBError::UniqueViolation)
                            } else {
                                Err(BaseDBError::BaseError(sqlx::Error::Database(db_err)))
                            }
                        } else {
                            Err(BaseDBError::BaseError(sqlx::Error::Database(db_err)))
                        }
                    }
                    not_db_err => Err(BaseDBError::BaseError(not_db_err)),
                },
            };
        })
    }

    fn get_user_by_uuid<'a>(
        &'a self,
        uuid: sqlx::types::Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<models::User, BaseDBError>> + Send + 'a>> {
        Box::pin(async move {
            match sqlx::query_as::<Postgres, models::User>(
                "SELECT uuid, email, password_hash, created_at FROM users WHERE uuid = $1",
            )
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            {
                Ok(optional_user) => match optional_user {
                    Some(user) => Ok(user),
                    None => Err(BaseDBError::RowNotFound),
                },
                Err(err) => Err(BaseDBError::BaseError(err)),
            }
        })
    }

    fn is_healthy<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move { self.pool.acquire().await.is_ok() })
    }

    fn get_user_by_email<'a>(
        &'a self,
        email: String,
    ) -> Pin<Box<dyn Future<Output = Result<(sqlx::types::Uuid, String), BaseDBError>> + Send + 'a>>
    {
        #[derive(sqlx::FromRow)]
        struct ReturnType(sqlx::types::Uuid, String);

        Box::pin(async move {
            match sqlx::query_as::<Postgres, ReturnType>(
                "SELECT uuid, password_hash FROM users WHERE email = $1",
            )
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            {
                Ok(opt) => match opt {
                    Some(ReturnType(uuid, real_password_hash)) => Ok((uuid, real_password_hash)),
                    None => Err(BaseDBError::RowNotFound),
                },
                Err(error) => Err(BaseDBError::BaseError(error)),
            }
        })
    }
}
