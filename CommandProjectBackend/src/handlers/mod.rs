use crate::app_state::{
    AppState, ServiceStatus, Status,
    results::{AuthUserResult, CreateUserResult, Errors},
};
use axum::Json;
use axum::extract::{Json as JsonExtractor, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use logger::LogLevel;
use serde;
use serde_json::json;

#[derive(serde::Deserialize)]
pub(crate) struct UserBody {
    email: String,
    password: String,
}

pub(crate) async fn healthcheck(State(state): State<AppState>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    let mut total_available = 0u8;
    let statuses: Vec<ServiceStatus> = state.clone().check_services().await;
    for service_status in &statuses {
        match service_status.service_status {
            Status::Working | Status::Warning => total_available += 1,
            _ => {}
        }
    }

    state.log("Healthcheck requested", LogLevel::Info);
    (
        StatusCode::OK,
        Json(json!({
            "timestamp": timestamp,
            "Total services available": total_available,
            "Services": statuses,
        })),
    )
}

#[axum::debug_handler]
pub(crate) async fn create_user_handler(
    State(state): State<AppState>,
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    use CreateUserResult as CUR;
    match state.create_new_user(email, password).await {
        CUR::EmailUsed => (
            StatusCode::CONFLICT,
            Json(json!({"Error": "Email is already user"})),
        ),
        CUR::BaseError(error) => match error {
            Errors::AuthServiceInternalError
            | Errors::NotFound
            | Errors::AuthServiceUnaccessible => (
                StatusCode::BAD_GATEWAY,
                Json(json!({"Error": "Auth service is not working properly"})),
            ),
            Errors::SelfInternalError(json_info) => (StatusCode::INTERNAL_SERVER_ERROR, json_info),
        },
        CUR::UserCreated { token } => (StatusCode::OK, Json(json!({"Token": token}))),
    }
}

pub(crate) async fn auth_user_handler(
    State(state): State<AppState>,
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    use AuthUserResult as AUR;
    match state.get_user_token(email, password).await {
        AUR::NoUserFound => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"Error": "No user with such email or password"})),
        ),
        AUR::BaseError(error) => match error {
            Errors::AuthServiceInternalError
            | Errors::NotFound
            | Errors::AuthServiceUnaccessible => (
                StatusCode::BAD_GATEWAY,
                Json(json!({"Error": "Auth service is not working properly"})),
            ),
            Errors::SelfInternalError(json_info) => (StatusCode::INTERNAL_SERVER_ERROR, json_info),
        },
        AUR::UserAuthenticated { token } => (StatusCode::OK, Json(json!({"Token": token}))),
    }
}
