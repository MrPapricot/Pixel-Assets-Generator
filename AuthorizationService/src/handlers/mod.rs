use crate::app_state::AppState;
use crate::database_adapter::{DBAdapter, custom_db_error::BaseDBError};
use crate::logger::LogLevel;
use axum::extract::{Json as JsonExtractor, State};
use axum::response::{IntoResponse, Json};
use serde;
use serde_json;

#[derive(serde::Deserialize)]
pub(crate) struct CreateUserBody {
    email: String,
    password_hash: String,
}

pub(crate) async fn default_handler(State(state): State<AppState>) -> impl IntoResponse {
    state.log("Called default handler", LogLevel::Info);
    Json(serde_json::json!({"hello": "what"}))
}

pub(crate) async fn create_user_handler(
    State(state): State<AppState>,
    JsonExtractor(CreateUserBody {
        email,
        password_hash,
    }): JsonExtractor<CreateUserBody>,
) -> impl IntoResponse {
    match state.create_new_user(email, password_hash).await {
        Ok(uuid) => {
            let token = state.get_jwt_token(uuid);
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({
                    "Status": "New user created",
                    "token": token,
                })),
            )
        }
        Err(base_db_err) => {
            use BaseDBError as BDE;
            match base_db_err {
                BDE::UniqueViolation => (
                    axum::http::StatusCode::CONFLICT,
                    Json(serde_json::json!({"Error": "This email is already used"})),
                ),
                BDE::BaseError(_) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"Error": "Internal Error"})),
                ),
                _ => unreachable!(),
            }
        }
    }
}
