use auth_rpc::auth::{
    AuthUserResult, CreateUserResult, HealthResult, UserBody, auth_client::AuthClient,
};
use tonic::{Request, Response, Status};

async fn create_grpc_client(addr: tonic::transport::Uri) -> AuthClient<tonic::transport::Channel> {
    let channel = tonic::transport::Channel::builder(addr)
        .connect()
        .await
        .expect("Should not happened");

    AuthClient::new(channel)
}

#[derive(Clone)]
pub struct AuthAdapter {
    client: AuthClient<tonic::transport::Channel>,
}

impl AuthAdapter {
    pub async fn new(addr: tonic::transport::Uri) -> Self {
        AuthAdapter {
            client: create_grpc_client(addr).await,
        }
    }

    pub async fn create_user(
        &mut self,
        email: String,
        password: String,
    ) -> Result<Response<CreateUserResult>, Status> {
        self.client
            .create_new_user(Request::new(UserBody { email, password }))
            .await
    }

    pub async fn auth_user(
        &mut self,
        email: String,
        password: String,
    ) -> Result<Response<AuthUserResult>, Status> {
        self.client
            .auth_user(Request::new(UserBody { email, password }))
            .await
    }

    pub async fn health(&mut self) -> Result<Response<HealthResult>, Status> {
        self.client.health(Request::new(())).await
    }
}
