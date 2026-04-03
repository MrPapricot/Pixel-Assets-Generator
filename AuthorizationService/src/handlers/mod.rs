use crate::app_state::AppState;
use crate::database_adapter::DBAdapter;
use crate::logger::LogLevel;
use axum::extract::State;
use axum::response::{IntoResponse, Json};
use serde_json;

pub(crate) async fn default_handler<DB: DBAdapter>(
    State(state): State<AppState<DB>>,
) -> impl IntoResponse {
    state.log("Called default handler", LogLevel::Info);
    Json(serde_json::json!({"hello": "what"}))
}
