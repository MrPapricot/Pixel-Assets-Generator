use axum::http::StatusCode;
use axum::routing::get;
use std::env;

mod app_state;
mod database_adapter;
mod handlers;
mod logger;
mod postgres_database_adapter;
mod simple_logger;

use logger::{LogLevel, Logger};
use simple_logger::SimpleLogger;

use app_state::AppState;

use database_adapter::DBAdapter;
use log::log;
use postgres_database_adapter::PostgresDBAdapter;

fn create_api_key_whitelist<L: Logger + 'static, DB: DBAdapter>(
    state: AppState<L, DB>,
) -> impl Fn(
    axum::extract::Request,
    axum::middleware::Next,
) -> std::pin::Pin<
    Box<dyn Future<Output = Result<axum::response::Response, StatusCode>> + Send>,
> + Clone
+ Send
+ 'static {
    let real_api_key = env::var("API_KEY").unwrap_or_else(|_| {
        state.get_logger().log(
            "No API_KEY provided through environment",
            LogLevel::CriticalError,
        );
        panic!();
    });
    let logger = state.get_logger();
    move |request, next| {
        let real_api_key = real_api_key.clone();
        let logger = logger.clone();
        Box::pin(async move {
            let user_api_key = request
                .headers()
                .get("x-api-key")
                .and_then(|value| value.to_str().ok());
            match user_api_key {
                Some(key) => {
                    if key == real_api_key {
                        Ok(next.run(request).await)
                    } else {
                        logger.log(
                            "Unauthorized access attempt (Wrong API key)",
                            LogLevel::Warning,
                        );
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
                None => {
                    logger.log(
                        "Unauthorized access attempt (No API key)",
                        LogLevel::Warning,
                    );
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let logger = SimpleLogger::new();

    let database_host = env::var("DB_HOST").unwrap_or("localhost".to_string());
    let database_port = match env::var("DB_PORT") {
        Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
            logger.log("DB_PORT from ENV must be a number", LogLevel::CriticalError);
            panic!();
        }),
        Err(err) => 5432,
    };
    let database_name = env::var("DB_NAME").unwrap_or("TestDB".to_string());
    let database_min_connections = match env::var("DB_MIN_CONNECTIONS") {
        Ok(port) => port.parse::<u32>().unwrap_or_else(|_| {
            logger.log(
                "DB_MIN_CONNECTIONS from ENV must be a number",
                LogLevel::CriticalError,
            );
            panic!();
        }),
        Err(err) => 1,
    };
    let database_max_connections = match env::var("DB_MAX_CONNECTIONS") {
        Ok(port) => port.parse::<u32>().unwrap_or_else(|_| {
            logger.log(
                "DB_MAX_CONNECTIONS from ENV must be a number",
                LogLevel::CriticalError,
            );
            panic!();
        }),
        Err(err) => 5,
    };
    let database_user = env::var("DB_USER").unwrap_or("postgres".to_string());
    let database_user_password = env::var("DB_USER_PASSWORD").unwrap_or("1234".to_string());

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = match env::var("SERVER_PORT") {
        Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
            logger.log(
                "SERVER_PORT from ENV must be a number",
                LogLevel::CriticalError,
            );
            panic!();
        }),
        Err(err) => 8080,
    };

    let db_url = format!(
        "postgresql://{database_user}:{database_user_password}@{database_host}:{database_port}/{database_name}"
    );
    let db_adapter = PostgresDBAdapter::connect(
        db_url.as_str(),
        database_min_connections,
        database_max_connections,
    )
    .await;
    let db_adapter = match db_adapter {
        Ok(adapter) => adapter,
        Err(error) => {
            logger.log(
                format!("Error connecting to database: {}", error.to_string()).as_str(),
                LogLevel::CriticalError,
            );
            panic!();
        }
    };

    logger.log(format!("Successfully connected to database \"{database_name}\"").as_str(), LogLevel::Info);

    let state = AppState::new(logger, db_adapter);

    let listener = tokio::net::TcpListener::bind(format!("{server_host}:{server_port}"))
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/health_check", get(handlers::health_check))
        .layer(axum::middleware::from_fn(create_api_key_whitelist(
            state.clone(),
        )))
        .with_state(state.clone());

    state.get_logger().log(
        format!("Start serving at http://{server_host}:{server_port}").as_str(),
        LogLevel::Info,
    );

    axum::serve(listener, app).await.unwrap();
}
