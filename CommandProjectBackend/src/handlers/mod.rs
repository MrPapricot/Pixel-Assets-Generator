use axum::extract::State;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use crate::app_state::AppState;
use crate::logger::LogLevel;

pub(crate) async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();
    state.log("Healthcheck requested", LogLevel::Info);
    (
        StatusCode::OK,
        Json(json!({
            "timestamp": timestamp,
            "Total services available": 0
        })),
    )
}
