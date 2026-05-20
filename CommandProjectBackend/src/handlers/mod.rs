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

use utoipa;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct UserBody {
    email: String,
    password: String,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(healthcheck, create_user_handler, auth_user_handler),
    tags(
        (name="TESTING", description="Для тестирования. Не использовать"),
        (name="API", description="Для обычного использования"),
    )
)]
pub struct ApiDoc;

#[derive(utoipa::ToSchema, serde::Serialize)]
pub(crate) struct HealthBody {
    #[serde(rename = "Services")]
    services: Vec<ServiceStatus>,

    #[serde(rename = "Total services available")]
    #[schema(example = 1)]
    total_available: u8,
    #[schema(example = "2026-05-20T12:24:25.488746300+00:00")]
    timestamp: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tags = ["TESTING"],
    description = "Проверяет работоспособность всех сервисов, и возвращает отчет",
    responses(
        (status = 200, description="Возвращает работоспособность всех сервисов", body=HealthBody)
    )
)]
pub(crate) async fn healthcheck(State(state): State<AppState>) -> (StatusCode, Json<HealthBody>) {
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
        Json(HealthBody {
            timestamp,
            total_available,
            services: statuses,
        }),
    )
}

#[derive(utoipa::ToSchema, serde::Serialize)]
struct UserSuccess {
    token: String,
}

#[utoipa::path(
    post,
    path = "/create_user",
    tags = ["API"],
    request_body = UserBody,
    description = "Создает пользователя с переданной почтой и паролем",
    responses(
        (status = 200, description = "Возвращает токен", body = UserSuccess),
        (status = 409, description = "Email уже используется", body = String),
        (status = 500, description = "Внутренняя ошибка сервиса", body = String),
        (status = 502, description = "Внешнии сервисы не работают", body = String),
    )
)]
#[axum::debug_handler]
pub(crate) async fn create_user_handler(
    State(state): State<AppState>,
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    use CreateUserResult as CUR;
    match state.create_new_user(email, password).await {
        CUR::EmailUsed => (StatusCode::CONFLICT, Json(json!("Email is already used"))),
        CUR::BaseError(error) => match error {
            Errors::AuthServiceInternalError
            | Errors::NotFound
            | Errors::AuthServiceUnaccessible => (
                StatusCode::BAD_GATEWAY,
                Json(json!(
                    "Some services are not working properly. Try again later"
                )),
            ),
            Errors::SelfInternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!("Something went wrong. Try again later")),
            ),
        },
        CUR::UserCreated { token } => (StatusCode::OK, Json(json!({"Token": token}))),
    }
}

#[utoipa::path(
    post,
    path = "/auth_user",
    tags = ["API"],
    request_body = UserBody,
    description = "Ищет пользователя по переданной почте и паролю",
    responses(
        (status = 200, description = "Возвращает токен", body = UserSuccess),
        (status = 401, description = "Пользователя с таким email и паролем нет", body = String),
        (status = 500, description = "Внутренняя ошибка сервиса", body = String),
        (status = 502, description = "Внешнии сервисы не работают", body = String),
    )
)]
pub(crate) async fn auth_user_handler(
    State(state): State<AppState>,
    JsonExtractor(UserBody { email, password }): JsonExtractor<UserBody>,
) -> impl IntoResponse {
    use AuthUserResult as AUR;
    match state.get_user_token(email, password).await {
        AUR::NoUserFound => (
            StatusCode::UNAUTHORIZED,
            Json(json!("No user with such email or password")),
        ),
        AUR::BaseError(error) => match error {
            Errors::AuthServiceInternalError
            | Errors::NotFound
            | Errors::AuthServiceUnaccessible => (
                StatusCode::BAD_GATEWAY,
                Json(json!("Auth service is not working properly")),
            ),
            Errors::SelfInternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!("Something went wrong. Try again later")),
            ),
        },
        AUR::UserAuthenticated { token } => (StatusCode::OK, Json(json!({"Token": token}))),
    }
}
