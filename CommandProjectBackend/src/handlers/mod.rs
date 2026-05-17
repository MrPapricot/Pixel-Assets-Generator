use crate::app_state::{AppState, ServiceStatus, Status, CreateUserBody};
use axum::Json;
use axum::extract::{Json as JsonExtractor, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use logger::LogLevel;
use serde_json::json;

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
    JsonExtractor(CreateUserBody {
        email,
        password_hash,
    }): JsonExtractor<CreateUserBody>,
) -> impl IntoResponse {
    state.create_new_user(email, password_hash).await
}
