use axum::routing::get;
use std::env;

mod app_state;
mod handlers;
mod logger;
mod simple_logger;

use logger::{LogLevel, Logger};
use simple_logger::SimpleLogger;

use app_state::AppState;

#[tokio::main]
async fn main() {
    let logger = SimpleLogger::new();

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = match env::var("SERVER_PORT") {
        Ok(port) => port.parse::<u16>().unwrap_or_else(|_| {
            logger.log(
                "SERVER_PORT from ENV must be a number",
                LogLevel::CriticalError,
            );
            panic!();
        }),
        Err(_) => 8080,
    };

    let state = AppState::new(Box::new(logger));

    let listener = match tokio::net::TcpListener::bind(format!("{server_host}:{server_port}")).await {
        Ok(listener) => listener,
        Err(error) => {
            state.log(format!("Unable to listen to {server_host}:{server_port}; Error occurred: {error}").as_str(), LogLevel::CriticalError);
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
