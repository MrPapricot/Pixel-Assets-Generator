use axum::routing::get;
use std::env;
use std::sync::{Arc, Mutex};

mod app_state;
mod handlers;

use logger::{LogLevel, Logger, simple_logger::SimpleLogger};

use app_state::AppState;

#[tokio::main]
async fn main() {
    let logger = SimpleLogger::new();

    let server_host = {
        const TARGET_VAR: &str = "SERVER_HOST";
        env::var(TARGET_VAR).unwrap_or_else(|_| {
            const DEFAULT_VALUE: &str = "localhost";
            logger.log(format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(), LogLevel::Warning);
            DEFAULT_VALUE.to_string()
        })
    };
    
    let server_port = {
        const TARGET_VAR: &str = "SERVER_PORT";
        const DEFAULT_VALUE: u16 = 8080u16;
        match env::var(TARGET_VAR) {
            Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
                logger.log(
                    format!("\"{TARGET_VAR}\" must be a number. Using default value: \"{DEFAULT_VALUE}\"").as_str(),
                    LogLevel::Warning,
                );
                DEFAULT_VALUE
            }),
            Err(_) => {
                logger.log(format!("\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\"").as_str(), LogLevel::Warning);
                DEFAULT_VALUE
            },
        }
    };

    let state = AppState::new(Arc::new(Mutex::new(logger)));

    let listener = match tokio::net::TcpListener::bind(format!("{server_host}:{server_port}")).await
    {
        Ok(listener) => listener,
        Err(error) => {
            state.log(
                format!("Unable to listen to {server_host}:{server_port}; Error occurred: {error}")
                    .as_str(),
                LogLevel::CriticalError,
            );
            panic!()
        }
    };

    let app = axum::Router::new()
        .route("/health_check", get(handlers::health_check))
        .with_state(state.clone());

    state.log(
        format!("Start serving at http://{server_host}:{server_port}").as_str(),
        LogLevel::Info,
    );

    axum::serve(listener, app).await.unwrap();
}

// TODO Сделать подсос к сервису авторизации
