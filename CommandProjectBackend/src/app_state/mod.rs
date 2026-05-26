use crate::rpc_implementation::AuthAdapter;
use auth_rpc::auth::{
    AuthUserError, CreateUserError, HealthStatus, ServerError, auth_user_result, TokenValidityStatus,
    create_user_result, token_validity,
};
use axum::Json;
use logger::{LogLevel, Logger};
use std::sync::{Arc, Mutex};

use crate::app_state::results::Errors;

pub(crate) mod results {
    use auth_rpc::auth::TokenValidityStatus;

    use super::Json;

    pub(crate) enum Errors {
        AuthServiceInternalError,
        SelfInternalError(Json<serde_json::Value>),
        AuthServiceUnaccessible,
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

    pub(crate) enum CheckToken {
        TokenStatus { status: TokenValidityStatus },
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
    "Message": "Database responds slow"
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
    auth_adapter: Arc<Mutex<AuthAdapter>>,
}

impl AppState {
    pub fn new(logger: Arc<Mutex<dyn Logger>>, auth_adapter: Arc<Mutex<AuthAdapter>>) -> AppState {
        AppState {
            logger: logger.clone(),
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
        let mut auth_adapter = self
            .auth_adapter
            .lock()
            .expect("Poisoned. Should not happed")
            .clone();
        responses.spawn(async move {
            match auth_adapter.health().await {
                Ok(response) => {
                    let health_result = response.into_inner();
                    let status;
                    match health_result.status {
                        res if res == HealthStatus::Ok as i32 => status = Status::Working,
                        res if res == HealthStatus::Warning as i32 => status = Status::Warning,
                        res if res == HealthStatus::Error as i32 => status = Status::Error,
                        _ => unreachable!(),
                    }
                    ServiceStatus {
                        service_name: "Auth Service".to_string(),
                        service_status: status,
                        message: health_result.message,
                    }
                }
                Err(_) => ServiceStatus {
                    service_name: "Auth Service".to_string(),
                    service_status: Status::NotFound,
                    message: None,
                },
            }
        });
        responses.join_all().await
    }

    pub async fn create_new_user(
        &self,
        email: String,
        password: String,
    ) -> results::CreateUserResult {
        let mut auth_adapter = self
            .auth_adapter
            .lock()
            .expect("Poisoned. Should not happed")
            .clone();
        match auth_adapter.create_user(email, password).await {
            Ok(response) => {
                let user_result = response.into_inner();
                match user_result.result {
                    Some(result) => {
                        use create_user_result::Result as res;
                        match result {
                            res::Token(token) => results::CreateUserResult::UserCreated { token },
                            res::Error(error) => match error {
                                err if err == CreateUserError::EmailUsed as i32 => {
                                    results::CreateUserResult::EmailUsed
                                }
                                _ => unreachable!(),
                            },
                            res::ServerError(server_error) => match server_error {
                                err if (err == ServerError::DbError as i32)
                                    | (err == ServerError::InternalServerError as i32) =>
                                {
                                    results::CreateUserResult::BaseError(
                                        Errors::AuthServiceInternalError,
                                    )
                                }
                                _ => unreachable!(),
                            },
                        }
                    }
                    None => unreachable!(),
                }
            }
            Err(_) => results::CreateUserResult::BaseError(Errors::AuthServiceUnaccessible),
        }
    }

    pub async fn get_user_token(&self, email: String, password: String) -> results::AuthUserResult {
        let mut auth_adapter = self
            .auth_adapter
            .lock()
            .expect("Poisoned. Should not happed")
            .clone();
        match auth_adapter.auth_user(email, password).await {
            Ok(response) => {
                let user_result = response.into_inner();
                match user_result.result {
                    Some(result) => {
                        use auth_user_result::Result as res;
                        match result {
                            res::Token(token) => {
                                results::AuthUserResult::UserAuthenticated { token }
                            }
                            res::Error(error) => match error {
                                err if err == AuthUserError::UserNotFound as i32 => {
                                    results::AuthUserResult::NoUserFound
                                }
                                _ => unreachable!(),
                            },
                            res::ServerError(server_error) => match server_error {
                                err if (err == ServerError::DbError as i32)
                                    | (err == ServerError::InternalServerError as i32) =>
                                {
                                    results::AuthUserResult::BaseError(
                                        Errors::AuthServiceInternalError,
                                    )
                                }
                                _ => unreachable!(),
                            },
                        }
                    }
                    None => unreachable!(),
                }
            }
            Err(_) => {
                self.log("Auth Service unaccessible", LogLevel::Error);
                results::AuthUserResult::BaseError(Errors::AuthServiceUnaccessible)
            }
        }
    }

    pub async fn check_token(&self, token: String) -> results::CheckToken {
        let mut auth_adapter = self
            .auth_adapter
            .lock()
            .expect("Poisoned. Should not happed")
            .clone();
        match auth_adapter.check_token(token).await {
            Ok(response) => {
                let user_result = response.into_inner();
                match user_result.result {
                    Some(result) => {
                        use token_validity::Result as res;
                        use TokenValidityStatus as TVS;
                        match result {
                            res::Status(status) => match status {
                                status if status == TVS::InvalidFormat as i32 => results::CheckToken::TokenStatus { status: TVS::InvalidFormat },
                                status if status == TVS::Expired as i32 => results::CheckToken::TokenStatus { status: TVS::Expired },
                                status if status == TVS::Valid as i32 => results::CheckToken::TokenStatus { status: TVS::Valid },
                                status if status == TVS::Invalid as i32 => results::CheckToken::TokenStatus { status: TVS::Invalid },
                                _ => unreachable!()
                            }
                            res::ServerError(server_error) => match server_error {
                                err if (err == ServerError::DbError as i32)
                                    | (err == ServerError::InternalServerError as i32) =>
                                {
                                    results::CheckToken::BaseError(
                                        Errors::AuthServiceInternalError,
                                    )
                                }
                                _ => unreachable!(),
                            },
                        }
                    }
                    None => unreachable!(),
                }
            }
            Err(_) => {
                self.log("Auth Service unaccessible", LogLevel::Error);
                results::CheckToken::BaseError(Errors::AuthServiceUnaccessible)
            }
        }
    }
}
