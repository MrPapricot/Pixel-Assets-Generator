use crate::app_state::{AppState, ServiceStatus, Status};
use logger::LogLevel;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use reqwest;

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