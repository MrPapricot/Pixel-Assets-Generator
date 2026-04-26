use std::env;
use std::sync::{Arc, Mutex};

mod app_state;
mod database_adapter;
mod handlers;
mod jwt_token_manager;
mod logger;
mod postgres_database_adapter;
mod simple_logger;

use logger::{LogLevel, Logger};
use simple_logger::SimpleLogger;

use crate::postgres_database_adapter::PostgresDBAdapter;

#[allow(unused_imports)]
use axum::routing::{delete, get, post, put};

use app_state::AppState;

// Главная точка входа в сервис авторизации
#[tokio::main]
async fn main() {
    let logger = SimpleLogger::new();

    let database_host = env::var("DB_HOST").unwrap_or("localhost".to_string());
    let database_port = match env::var("DB_PORT") {
        Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
            logger.log("DB_PORT from ENV must be a number", LogLevel::CriticalError);
            panic!();
        }),
        Err(_) => 5432,
    };
    let database_user = env::var("DB_USER").unwrap_or("postgres".to_string());
    let database_user_password = env::var("DB_USER_PASSWORD").unwrap_or("1234".to_string());
    let database_name = env::var("DB_NAME").unwrap_or("TestDB".to_string());
    let database_url = format!(
        "postgresql://{database_user}:{database_user_password}@{database_host}:{database_port}/{database_name}"
    );

    let service_host = env::var("AUTH_HOST").unwrap_or("localhost".to_string());
    let service_port = match env::var("AUTH_PORT") {
        Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
            logger.log(
                "AUTH_PORT from ENV must be a number",
                LogLevel::CriticalError,
            );
            panic!();
        }),
        Err(_) => 8070,
    };

    logger.log(
        format!("Connecting to DB on url {database_url}").as_str(),
        LogLevel::Info,
    );

    let database_adapter = match PostgresDBAdapter::connect(database_url).await {
        Ok(adapter) => adapter,
        Err(error) => {
            logger.log(
                format!("Error connecting to Database. Error occurred: {error}").as_str(),
                LogLevel::CriticalError,
            );
            panic!();
        }
    };
    logger.log("Successfully connected to database", LogLevel::Info);

    let state = AppState::new(
        Arc::new(Mutex::new(logger)),
        Arc::new(database_adapter.clone()),
    );

    let app = axum::routing::Router::new()
        .route("/", get(handlers::default_handler))
        .route("/new_user", post(handlers::create_user_handler))
        .route("/get_user", get(handlers::get_user_handler))
        .route("/healthcheck", get(handlers::healthcheck))
        .with_state(state.clone());

    let listener =
        match tokio::net::TcpListener::bind(format!("{service_host}:{service_port}")).await {
            Ok(listener) => listener,
            Err(error) => {
                state.log(
                    format!(
                        "Unable to listen to {service_host}:{service_port}; Error occurred: {error}"
                    )
                    .as_str(),
                    LogLevel::CriticalError,
                );
                panic!()
            }
        };

    state.log(
        format!("Start serving at http://{service_host}:{service_port}").as_str(),
        LogLevel::Info,
    );

    axum::serve(listener, app).await.unwrap();
}

// TODO Опционально перейти на Protobuf, но в принципе и так норм
