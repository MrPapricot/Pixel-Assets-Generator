use axum::Json;
use logger::{LogLevel, Logger};
use reqwest;
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

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
    NotActive,
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
    #[serde(rename = "JSONMessage")]
    pub(crate) service_json_output: Option<serde_json::Value>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    services: Arc<RwLock<HashMap<Services, ServiceData>>>,
}

impl AppState {
    pub fn new(
        logger: Arc<Mutex<dyn Logger>>,
        services: Arc<RwLock<HashMap<Services, ServiceData>>>,
    ) -> AppState {
        AppState {
            logger: logger.clone(),
            services: services.clone(),
        }
    }

    pub fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }

    pub async fn check_services(&self) -> Vec<ServiceStatus> {
        let services: HashMap<Services, ServiceData> =
            (*self.services.read().expect("Poisoned, Should not happen")).clone();
        let mut responses = tokio::task::JoinSet::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Should not fail");
        for service in services.values() {
            let client = client.clone();
            let service = service.clone();
            responses.spawn(async move {
                let response = client
                    .get(format!(
                        "http://{}:{}/health",
                        service.service_host, service.service_port
                    ))
                    .send()
                    .await;
                let service_name: String = service.service_name;
                let status: Status;
                let output: Option<serde_json::Value>;
                if let Ok(response) = response {
                    status = {
                        use reqwest::StatusCode as SC;
                        match response.status() {
                            SC::OK => Status::Working,
                            SC::NOT_FOUND => Status::NotFound,
                            _ => Status::Warning,
                        }
                    };
                    output = response.json::<serde_json::Value>().await.ok();
                } else {
                    status = Status::NotActive;
                    output = None;
                }
                ServiceStatus {
                    service_name,
                    service_status: status,
                    service_json_output: output,
                }
            });
        }
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
                    .json(&AuthUserBody {
                        email,
                        password,
                    })
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
