use std::env;
use std::sync::{Arc, Mutex};

mod app_state;
mod database_adapter;
mod handlers;
mod jwt_token_manager;
mod postgres_database_adapter;
use logger::{Logger, LogLevel, simple_logger::SimpleLogger};

use crate::postgres_database_adapter::PostgresDBAdapter;

#[allow(unused_imports)]
use axum::routing::{delete, get, post, put};

use app_state::AppState;

// Главная точка входа в сервис авторизации
#[tokio::main]
async fn main() {
    let logger = SimpleLogger::new();

    let database_host = {
        const TARGET_VAR: &str = "DB_HOST";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "localhost";
            logger.log(
                format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                LogLevel::Warning,
            );
            DEFAULT_VALUE.to_string()
        })
    };

    let database_port = {
        const TARGET_VAR: &str = "DB_PORT";
        const DEFAULT_VALUE: u16 = 5432;
        match env::var(TARGET_VAR) {
            Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
                logger.log(
                    format!("\"{TARGET_VAR}\" must be a number. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                    LogLevel::Warning,
                );
                DEFAULT_VALUE
            }),
            Err(_) => {
                logger.log(
                    format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                    LogLevel::Warning,
                );
                DEFAULT_VALUE
            },
        }
    };

    let database_user = {
        const TARGET_VAR: &str = "DB_USER";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "postgres";
            logger.log(
                format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                LogLevel::Warning,
            );
            DEFAULT_VALUE.to_string()
        })
    };

    let database_user_password = {
        const TARGET_VAR: &str = "DB_USER_PASSWORD";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "1234";
            logger.log(
                format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                LogLevel::Warning,
            );
            DEFAULT_VALUE.to_string()
        })
    };

    let database_name = {
        const TARGET_VAR: &str = "DB_NAME";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "TestDB";
            logger.log(
                format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                LogLevel::Warning,
            );
            DEFAULT_VALUE.to_string()
        })
    };

    let database_url = format!(
        "postgresql://{database_user}:{database_user_password}@{database_host}:{database_port}/{database_name}"
    );

    let service_host = {
        const TARGET_VAR: &str = "AUTH_HOST";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "localhost";
            logger.log(
                format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                LogLevel::Warning,
            );
            DEFAULT_VALUE.to_string()
        })
    };

    let service_port = {
        const TARGET_VAR: &str = "AUTH_PORT";
        const DEFAULT_VALUE: u16 = 8070;
        match env::var(TARGET_VAR) {
            Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
                logger.log(
                    format!("\"{TARGET_VAR}\" must be a number. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                    LogLevel::Warning,
                );
                DEFAULT_VALUE
            }),
            Err(_) => {
                logger.log(
                    format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                    LogLevel::Warning,
                );
                DEFAULT_VALUE
            },
        }
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
// TODO Сделать кафку
