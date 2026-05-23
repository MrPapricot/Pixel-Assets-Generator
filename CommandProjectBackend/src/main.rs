use axum::routing::{get, post};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::{env, fs};

mod app_state;
mod handlers;
mod rpc_implementation;

use logger::{LogLevel, Logger, simple_logger::SimpleLogger};

use crate::app_state::{ServiceData, Services};
use crate::rpc_implementation::AuthAdapter;
use app_state::AppState;

use utoipa::OpenApi;
use utoipa_swagger_ui;

#[tokio::main]
async fn main() {
    match env::args().skip(1).next() {
        Some(arg) if arg == "GEN_API" => {
            let api = handlers::ApiDoc::openapi();
            let json = serde_json::to_string_pretty(&api).expect("❌ Failed to serialize OpenAPI");
            fs::write("openapi.json", json).expect("❌ Failed to write openapi.json");
            println!("✅ OpenAPI spec generated at openapi.json");
        }
        Some(arg) => {
            println!("{}", arg);
            println!("Unexpected arg");
            panic!()
        }
        None => {
            let logger = SimpleLogger::new();

            let server_host = {
                const TARGET_VAR: &str = "SERVER_HOST";
                env::var(TARGET_VAR).unwrap_or_else(|_| {
                    const DEFAULT_VALUE: &str = "localhost";
                    logger.log(
                        format!(
                            "\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\""
                        )
                            .as_str(),
                        LogLevel::Warning,
                    );
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

            let auth_host = {
                const TARGET_VAR: &str = "AUTH_HOST";
                env::var(TARGET_VAR).unwrap_or_else(|_| {
                    const DEFAULT_VALUE: &str = "localhost";
                    logger.log(
                        format!(
                            "\"{TARGET_VAR}\" is not defined. Using default value: \"{DEFAULT_VALUE}\""
                        )
                            .as_str(),
                        LogLevel::Warning,
                    );
                    DEFAULT_VALUE.to_string()
                })
            };

            let auth_port = {
                const TARGET_VAR: &str = "AUTH_PORT";
                const DEFAULT_VALUE: u16 = 8070u16;
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

            let services: HashMap<Services, ServiceData> = HashMap::from([(
                Services::Auth,
                ServiceData::new("Auth Service".to_string(), auth_host.clone(), auth_port.clone()),
            )]);

            let auth_adapter = AuthAdapter::new(format!("http://{auth_host}:{auth_port}").parse().expect("Should be valid URI")).await;

            let state = AppState::new(
                Arc::new(Mutex::new(logger)),
                Arc::new(RwLock::new(services)),
                Arc::new(Mutex::new(auth_adapter)),
            );

            let listener = match tokio::net::TcpListener::bind(format!(
                "{server_host}:{server_port}"
            ))
            .await
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
                .route("/health", get(handlers::healthcheck))
                .route("/create_user", post(handlers::create_user_handler))
                .route("/auth_user", post(handlers::auth_user_handler))
                .merge(
                    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                        .url("/api-docs/openapi.json", handlers::ApiDoc::openapi()),
                )
                .with_state(state.clone());

            state.log(
                format!("Start serving at http://{server_host}:{server_port}").as_str(),
                LogLevel::Info,
            );

            axum::serve(listener, app).await.unwrap();
        }
    }
}
