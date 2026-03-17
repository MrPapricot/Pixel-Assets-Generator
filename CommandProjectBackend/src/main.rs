use axum::http::StatusCode;
use axum::routing::get;

mod handlers;
mod logger;
mod simple_logger;
mod app_state;

use logger::{LogLevel, Logger};

use simple_logger::SimpleLogger;
use crate::app_state::AppState;

fn create_api_key_whitelist<L: Logger>(state: &AppState<L>) -> impl Fn(
    axum::extract::Request,
    axum::middleware::Next,
) -> std::pin::Pin<Box<dyn Future<Output = Result<axum::response::Response, StatusCode>> + Send>> + Clone + Send + 'static {
    let real_api_key = std::env::var("API_KEY").unwrap_or_else(|_| {
        state.get_logger().log("No API_KEY provided through environment", LogLevel::CriticalError);
        panic!();
    });
    move |request, next| {
        let real_api_key = real_api_key.clone();
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
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
                None => Err(StatusCode::UNAUTHORIZED),
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let host = "0.0.0.0";
    let port = 8080;
    let state = AppState::new(SimpleLogger::new());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    let app = axum::Router::new()
        .route("/health_check", get(handlers::health_check))
        .layer(axum::middleware::from_fn(create_api_key_whitelist(&state)))
        .with_state(state.clone());
    state.get_logger().log(format!("Start serving at {host}:{port}").as_str(), LogLevel::Info);
    axum::serve(listener, app).await.unwrap();
}
