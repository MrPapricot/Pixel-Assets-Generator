use std::hint::unreachable_unchecked;

use crate::app_state::{AppState, Error};
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
pub(crate) struct UserBody {
    email: String,
    password: String,
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
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    match state.create_new_user(email, password).await {
        Ok(uuid) => {
            let token = state.get_jwt_token(uuid);
            (
                StatusCode::OK,
                Json(json!({
                    "token": token,
                })),
            )
        }
        Err(base_db_err) => {
            use BaseDBError as BDE;
            match base_db_err {
                Error::DBError(BDE::UniqueViolation) => (
                    StatusCode::CONFLICT,
                    Json(json!({"Error": "This email is already used"})),
                ),
                Error::DBError(BDE::BaseError(_)) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"Error": "Internal Error"})),
                ),
                Error::HashingError => {
                    state.log("Hashing error happened", LogLevel::Error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"Error": "Internal Error"})),
                    )
                }
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) async fn healthcheck(State(state): State<AppState>) -> impl IntoResponse {
    if state.check_database_health().await {
        (StatusCode::OK, Json(json!({"Status": "Active"})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"Error": "Connection to Database is lost"})),
        )
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
                                LogLevel::Error,
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

pub(crate) async fn get_user_by_email_and_password_handler(
    State(state): State<AppState>,
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    match state.get_user_uuid(email, password).await {
        Ok(uuid) => (
            StatusCode::OK,
            Json(json!({"token": state.get_jwt_token(uuid)})),
        ),
        Err(error) => {
            use BaseDBError as BDE;
            match error {
                BDE::BaseError(err) => {
                    state.log(
                        format!("Error getting user from DB. Error is: {}", err).as_str(),
                        LogLevel::Error,
                    );
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"Error": "Internal Error"})),
                    )
                }
                BDE::RowNotFound | BDE::WrongPassword => (
                    StatusCode::NO_CONTENT,
                    Json(json!({"Error": "No user with such email and password"})),
                ),
                BDE::UniqueViolation => unsafe { unreachable_unchecked() },
            }
        }
    }
}
