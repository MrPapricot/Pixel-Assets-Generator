use crate::app_state::{AppState, Error};
use crate::database_adapter::custom_db_error::BaseDBError;
use auth_rpc::auth::{
    AuthUserError, AuthUserResult, CreateUserError, CreateUserResult, HealthResult, HealthStatus,
    ServerError, UserBody, auth_server::Auth, auth_user_result, create_user_result,
};
use tonic::{Request, Response, Status};

pub struct AuthService {
    pub state: AppState,
}

impl AuthService {
    pub fn new(state: AppState) -> Self {
        AuthService { state }
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn create_new_user(
        &self,
        request: Request<UserBody>,
    ) -> Result<Response<CreateUserResult>, Status> {
        let body = request.into_inner();
        match self.state.create_new_user(body.email, body.password).await {
            Ok(token) => Ok(Response::new(CreateUserResult {
                result: Some(create_user_result::Result::Token(token)),
            })),
            Err(err) => Ok(Response::new(CreateUserResult {
                result: match err {
                    Error::HashingError => Some(create_user_result::Result::ServerError(
                        ServerError::InternalServerError as i32,
                    )),
                    Error::DBError(db_err) => match db_err {
                        BaseDBError::UniqueViolation => Some(create_user_result::Result::Error(
                            CreateUserError::EmailUsed as i32,
                        )),
                        BaseDBError::BaseError(_) => Some(create_user_result::Result::ServerError(
                            ServerError::DbError as i32,
                        )),
                        BaseDBError::RowNotFound | BaseDBError::WrongPassword => unreachable!(),
                    },
                },
            })),
        }
    }

    async fn auth_user(
        &self,
        request: Request<UserBody>,
    ) -> Result<Response<AuthUserResult>, Status> {
        let body = request.into_inner();
        match self.state.get_user_uuid(body.email, body.password).await {
            Ok(token) => Ok(Response::new(AuthUserResult {
                result: Some(auth_user_result::Result::Token(token)),
            })),
            Err(err) => Ok(Response::new(AuthUserResult {
                result: match err {
                    Error::HashingError => Some(auth_user_result::Result::ServerError(
                        ServerError::InternalServerError as i32,
                    )),
                    Error::DBError(db_err) => match db_err {
                        BaseDBError::UniqueViolation => unreachable!(),
                        BaseDBError::BaseError(_) => Some(auth_user_result::Result::ServerError(
                            ServerError::DbError as i32,
                        )),
                        BaseDBError::RowNotFound | BaseDBError::WrongPassword => Some(
                            auth_user_result::Result::Error(AuthUserError::UserNotFound as i32),
                        ),
                    },
                },
            })),
        }
    }

    async fn health(&self, _request: Request<()>) -> Result<Response<HealthResult>, Status> {
        if self.state.check_database_health().await {
            Ok(Response::new(HealthResult {
                status: HealthStatus::Ok as i32,
                message: None,
            }))
        } else {
            Ok(Response::new(HealthResult {
                status: HealthStatus::Error as i32,
                message: Some(String::from("Database isn't working")),
            }))
        }
    }
}
