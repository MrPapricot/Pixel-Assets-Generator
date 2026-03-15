use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub(crate) async fn health_check() -> impl IntoResponse {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();
    (
        StatusCode::OK,
        Json(json!({
            "timestamp": timestamp,
            "Total services available": 0
        })),
    )
}