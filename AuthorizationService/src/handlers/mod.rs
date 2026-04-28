use crate::app_state::AppState;
use crate::database_adapter::custom_db_error::BaseDBError;
use axum::extract::{Json as JsonExtractor, State};
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Json};
use logger::LogLevel;
use serde;
use serde_json;
use serde_json::json;

#[derive(serde::Deserialize)]
pub(crate) struct CreateUserBody {
    email: String,
    password_hash: String,
}

// Хендлер чисто по приколу сделан (только для тестов)
pub(crate) async fn default_handler(State(state): State<AppState>) -> impl IntoResponse {
    state.log("Called default handler", LogLevel::Info);
    Json(json!({"hello": "what"}))
}

// Хендлер для создания пользователя, в случае если емайл уже используется возвращает ошибку Email
// already used
// Иначе возвращает токен
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
                StatusCode::OK,
                Json(json!({
                    "Status": "New user created",
                    "token": token,
                })),
            )
        }
        Err(base_db_err) => {
            use BaseDBError as BDE;
            match base_db_err {
                BDE::UniqueViolation => (
                    StatusCode::CONFLICT,
                    Json(json!({"Error": "This email is already used"})),
                ),
                BDE::BaseError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"Error": "Internal Error"})),
                ),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) async fn healthcheck(State(state): State<AppState>) -> impl IntoResponse {
    if state.check_database_health().await {
        (StatusCode::OK, Json(json!({"Status": "Active"})))
    }
    else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"Error": "Connection to Database is lost"})))
    }
}

pub(crate) async fn get_user_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let token = headers.get("Token");
    match token {
        None => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"Error": "No token provided"})),
        ),
        Some(token) => {
            let uuid = state.get_uuid_from_token(token.as_bytes());

            match uuid {
                Err(err) => {
                    use crate::jwt_token_manager::JWTDecodingError as JDE;
                    match err {
                        JDE::NoUUID => (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"Error": "JWT contains invalid data"})),
                        ),
                        JDE::InvalidToken => (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"Error": "Token is not a valid JWT"})),
                        ),
                        JDE::TokenExpired => (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"Error": "Token expired"})),
                        ),
                    }
                }
                Ok(uuid) => {
                    let user = state.get_user_by_uuid(uuid).await;
                    match user {
                        Ok(user) => (
                            StatusCode::OK,
                            Json(json!({"email": user.email, "created_at": user.created_at})),
                        ),
                        Err(BaseDBError::RowNotFound) => (
                            StatusCode::NO_CONTENT,
                            Json(json!({"Error": "No users found"})),
                        ),
                        Err(BaseDBError::BaseError(err)) => {
                            state.log(
                                format!("Error getting user from DB. Error is: {}", err).as_str(),
                                LogLevel::Warning,
                            );
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"Error": "Internal Error"})),
                            )
                        }
                        _ => {
                            state.log(format!("Reached unreachable code, when getting user by uuid, with uuid = {}", uuid).as_str(), LogLevel::CriticalError);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({"Error": "Should not be here"})),
                            )
                        }
                    }
                }
            }
        }
    }
}
