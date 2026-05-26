use crate::app_state::{AppState, Error};
use crate::database_adapter::custom_db_error::BaseDBError;
use auth_rpc::auth::{
    AuthUserError, AuthUserResult, CheckTokenBody, CreateUserError, CreateUserResult, HealthResult,
    HealthStatus, ServerError, TokenValidity, UserBody, auth_server::Auth, auth_user_result,
    create_user_result, token_validity
};
use logger::LogLevel;
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
        self.state.log("Health requested", LogLevel::Info);
        if self.state.check_database_health().await {
            Ok(Response::new(HealthResult {
                status: HealthStatus::Ok as i32,
                message: None,
            }))
        } else {
            self.state.log("Database isn't working", LogLevel::Error);
            Ok(Response::new(HealthResult {
                status: HealthStatus::Error as i32,
                message: Some(String::from("Database isn't working")),
            }))
        }
    }

    async fn check_token(
        &self,
        request: Request<CheckTokenBody>,
    ) -> Result<Response<TokenValidity>, Status> {
        let body = request.into_inner();
        self.state.log("Requested token check", LogLevel::Info);
        match self.state.check_token_validity(body.token).await {
            Ok(token_state) => {
                use crate::app_state::TokenValidity as STV;
                use auth_rpc::auth::TokenValidityStatus as ATV;

                let validity = match token_state {
                    STV::InvalidFormat => ATV::InvalidFormat as i32,
                    STV::Expired => ATV::Expired as i32,
                    STV::Valid => ATV::Valid as i32,
                    STV::Invalid => ATV::Invalid as i32,
                };

                Ok(Response::new(TokenValidity {
                    result: Some(token_validity::Result::Status(validity)),
                }))
            }
            Err(db_err) => {
                self.state.log(format!("Error checking token: {:?}", db_err).as_str(), LogLevel::Error);
                Ok(Response::new(TokenValidity {
                    result: Some(token_validity::Result::ServerError(ServerError::DbError as i32)),
                }))
            }
        }
    }
}
