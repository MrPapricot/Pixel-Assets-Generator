use auth_rpc::auth::HealthStatus;
use axum::Json;
use logger::{LogLevel, Logger};
use reqwest;
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::rpc_implementation::AuthAdapter;

use crate::app_state::results::{AuthUserResult, CreateUserResult, Errors};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum Services {
    Auth,
}

pub(crate) mod results {
    use super::Json;

    pub(crate) enum Errors {
        AuthServiceInternalError,
        SelfInternalError(Json<serde_json::Value>),
        AuthServiceUnaccessible,
        NotFound,
    }

    pub(crate) enum CreateUserResult {
        UserCreated { token: String },
        EmailUsed,
        BaseError(Errors),
    }

    pub(crate) enum AuthUserResult {
        UserAuthenticated { token: String },
        NoUserFound,
        BaseError(Errors),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct AuthUserBody {
    pub email: String,
    pub password: String,
}

#[derive(Clone)]
pub(crate) struct ServiceData {
    service_name: String,
    service_host: String,
    service_port: u16,
}

impl ServiceData {
    pub(crate) fn new(
        service_name: String,
        service_host: String,
        service_port: u16,
    ) -> ServiceData {
        ServiceData {
            service_name,
            service_host,
            service_port,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub(crate) enum Status {
    Working,
    Warning,
    Error,
    NotFound,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "Name": "Auth Service",
    "Status": "Warning",
    "JSONMessage": {
        "Warning": "Database responds slowly"
    }
}))]
pub(crate) struct ServiceStatus {
    #[serde(rename = "Name")]
    pub(crate) service_name: String,
    #[serde(rename = "Status")]
    pub(crate) service_status: Status,
    #[serde(rename = "Message")]
    pub(crate) message: Option<String>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    services: Arc<RwLock<HashMap<Services, ServiceData>>>,
    auth_adapter: Arc<Mutex<AuthAdapter>>,
}

impl AppState {
    pub fn new(
        logger: Arc<Mutex<dyn Logger>>,
        services: Arc<RwLock<HashMap<Services, ServiceData>>>,
        auth_adapter: Arc<Mutex<AuthAdapter>>,
    ) -> AppState {
        AppState {
            logger: logger.clone(),
            services: services.clone(),
            auth_adapter: auth_adapter.clone(),
        }
    }

    pub fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }

    pub async fn check_services(&mut self) -> Vec<ServiceStatus> {
        let mut responses = tokio::task::JoinSet::new();
        let mut auth_adapter = self.auth_adapter.lock().expect("Poisoned. Should not happed").clone();
        responses.spawn(async move {
            match auth_adapter.health().await {
                Ok(response) => {
                    let health_result = response.into_inner();
                    let status;
                    match health_result.status {
                        res if res == HealthStatus::Ok as i32 => status = Status::Working,
                        res if res == HealthStatus::Warning as i32 => status = Status::Warning,
                        res if res == HealthStatus::Error as i32 => status = Status::Error,
                        _ => unreachable!()
                    }
                    ServiceStatus {
                        service_name: "Auth Service".to_string(),
                        service_status: status,
                        message: health_result.message,
                    }
                }
                Err(_) => {
                    ServiceStatus {
                        service_name: "Auth Service".to_string(),
                        service_status: Status::NotFound,
                        message: None,
                    }
                }
            }
        });
        responses.join_all().await
    }

    pub async fn create_new_user(&self, email: String, password: String) -> CreateUserResult {
        match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Err(error) => {
                self.log(
                    format!("Error creating reqwest client: {}", error).as_str(),
                    LogLevel::Error,
                );
                CreateUserResult::BaseError(Errors::SelfInternalError(Json(
                    json!({"Error": "Something went wrong. Try again later"}),
                )))
            }
            Ok(client) => {
                let auth_service: ServiceData =
                    (*self.services.read().expect("Poisoned, Should not happen"))
                        .get(&Services::Auth)
                        .expect("Should not fail")
                        .clone();

                let endpoint = "new_user";

                let response = client
                    .post(format!(
                        "http://{}:{}/{endpoint}",
                        auth_service.service_host, auth_service.service_port
                    ))
                    .json(&AuthUserBody { email, password })
                    .send()
                    .await;
                if let Ok(response) = response {
                    match response.status() {
                        StatusCode::OK => {
                            #[derive(serde::Serialize, serde::Deserialize)]
                            struct UserCreatedBody {
                                token: String,
                            }
                            match response.json::<UserCreatedBody>().await {
                                Ok(body) => CreateUserResult::UserCreated { token: body.token },
                                Err(error) => {
                                    self.log(format!("Auth service returned unexpectable body on new_user request. Error is \"{:?}\"", error).as_str(), LogLevel::Error);
                                    CreateUserResult::BaseError(Errors::SelfInternalError(Json(
                                        json!({"Error": "Error decoding respose from Auth:create_new_user"}),
                                    )))
                                }
                            }
                        }
                        StatusCode::INTERNAL_SERVER_ERROR => {
                            CreateUserResult::BaseError(Errors::AuthServiceInternalError)
                        }
                        StatusCode::CONFLICT => CreateUserResult::EmailUsed,
                        StatusCode::NOT_FOUND => {
                            self.log(
                                format!("This endpoint is not found in auth service: {endpoint}")
                                    .as_str(),
                                LogLevel::CriticalError,
                            );
                            CreateUserResult::BaseError(Errors::NotFound)
                        }
                        code => {
                            self.log(
                                format!("Auth Service returned unexpectable status code: {}", code)
                                    .as_str(),
                                LogLevel::Error,
                            );
                            CreateUserResult::BaseError(Errors::SelfInternalError(Json(
                                json!({"Error": "Something went wrong. Try again later"}),
                            )))
                        }
                    }
                } else {
                    self.log("Auth service is not accessible", LogLevel::Error);
                    CreateUserResult::BaseError(Errors::AuthServiceUnaccessible)
                }
            }
        }
    }

    pub async fn get_user_token(&self, email: String, password: String) -> AuthUserResult {
        match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Err(error) => {
                self.log(
                    format!("Error creating reqwest client: {}", error).as_str(),
                    LogLevel::Error,
                );
                AuthUserResult::BaseError(Errors::SelfInternalError(Json(
                    json!({"Error": "Something went wrong. Try again later"}),
                )))
            }
            Ok(client) => {
                let auth_service: ServiceData =
                    (*self.services.read().expect("Poisoned, Should not happen"))
                        .get(&Services::Auth)
                        .expect("Should not fail")
                        .clone();

                let endpoint = "auth_user";

                let response = client
                    .post(format!(
                        "http://{}:{}/{endpoint}",
                        auth_service.service_host, auth_service.service_port
                    ))
                    .json(&AuthUserBody { email, password })
                    .send()
                    .await;
                if let Ok(response) = response {
                    match response.status() {
                        StatusCode::OK => {
                            #[derive(serde::Serialize, serde::Deserialize)]
                            struct UserCreatedBody {
                                token: String,
                            }
                            match response.json::<UserCreatedBody>().await {
                                Ok(body) => AuthUserResult::UserAuthenticated { token: body.token },
                                Err(error) => {
                                    self.log(format!("Auth service returned unexpectable body on new_user request. Error is \"{:?}\"", error).as_str(), LogLevel::Error);
                                    AuthUserResult::BaseError(Errors::SelfInternalError(Json(
                                        json!({"Error": "Error decoding respose from Auth:create_new_user"}),
                                    )))
                                }
                            }
                        }
                        StatusCode::INTERNAL_SERVER_ERROR => {
                            AuthUserResult::BaseError(Errors::AuthServiceInternalError)
                        }
                        StatusCode::NO_CONTENT => AuthUserResult::NoUserFound,
                        StatusCode::NOT_FOUND => {
                            self.log(
                                format!("This endpoint is not found in auth service: {endpoint}")
                                    .as_str(),
                                LogLevel::CriticalError,
                            );
                            AuthUserResult::BaseError(Errors::NotFound)
                        }
                        code => {
                            self.log(
                                format!("Auth Service returned unexpectable status code: {}", code)
                                    .as_str(),
                                LogLevel::Error,
                            );
                            AuthUserResult::BaseError(Errors::SelfInternalError(Json(
                                json!({"Error": "Something went wrong. Try again later"}),
                            )))
                        }
                    }
                } else {
                    self.log("Auth service is not accessible", LogLevel::Error);
                    AuthUserResult::BaseError(Errors::AuthServiceUnaccessible)
                }
            }
        }
    }
}
