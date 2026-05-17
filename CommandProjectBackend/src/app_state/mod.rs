use axum::http::StatusCode;
use logger::{LogLevel, Logger};
use reqwest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum Services {
    Auth,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CreateUserBody {
    pub email: String,
    pub password_hash: String,
}


#[derive(Clone)]
pub(crate) struct ServiceData {
    service_name: String,
    service_host: String,
    service_port: u16,
}

impl ServiceData {
    pub(crate) fn new(
        service_name: String,
        service_host: String,
        service_port: u16,
    ) -> ServiceData {
        ServiceData {
            service_name,
            service_host,
            service_port,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) enum Status {
    Working,
    Warning,
    NotActive,
    NotFound,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ServiceStatus {
    #[serde(rename = "Name")]
    pub(crate) service_name: String,
    #[serde(rename = "Status")]
    pub(crate) service_status: Status,
    #[serde(rename = "JSONMessage")]
    pub(crate) service_json_output: Option<serde_json::Value>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    services: Arc<RwLock<HashMap<Services, ServiceData>>>,
}

impl AppState {
    pub fn new(
        logger: Arc<Mutex<dyn Logger>>,
        services: Arc<RwLock<HashMap<Services, ServiceData>>>,
    ) -> AppState {
        AppState {
            logger: logger.clone(),
            services: services.clone(),
        }
    }

    pub fn log(&self, message: &str, log_level: LogLevel) {
        self.logger
            .lock()
            .expect("poisoned")
            .log(message, log_level);
    }

    pub async fn check_services(&self) -> Vec<ServiceStatus> {
        let services: HashMap<Services, ServiceData> =
            (*self.services.read().expect("Poisoned, Should not happen")).clone();
        let mut responses = tokio::task::JoinSet::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Should not fail");
        for service in services.values() {
            let client = client.clone();
            let service = service.clone();
            responses.spawn(async move {
                let response = client
                    .get(format!(
                        "http://{}:{}/health",
                        service.service_host, service.service_port
                    ))
                    .send()
                    .await;
                let service_name: String = service.service_name;
                let status: Status;
                let output: Option<serde_json::Value>;
                if let Ok(response) = response {
                    status = {
                        use reqwest::StatusCode as SC;
                        match response.status() {
                            SC::OK => Status::Working,
                            SC::NOT_FOUND => Status::NotFound,
                            _ => Status::Warning,
                        }
                    };
                    output = response.json::<serde_json::Value>().await.ok();
                } else {
                    status = Status::NotActive;
                    output = None;
                }
                ServiceStatus {
                    service_name,
                    service_status: status,
                    service_json_output: output,
                }
            });
        }
        responses.join_all().await
    }

    pub async fn create_new_user(
        &self,
        email: String,
        password_hash: String,
    ) -> (StatusCode, axum::Json<serde_json::Value>) {
        let services: HashMap<Services, ServiceData> =
            (*self.services.read().expect("Poisoned, Should not happen")).clone();
        let client: reqwest::Client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Should not fail");
        let auth_service: &ServiceData = services.get(&Services::Auth).expect("Should not fail");

        let response = client
            .post(format!(
                "http://{}:{}/new_user",
                auth_service.service_host, auth_service.service_port
            ))
            .json(&CreateUserBody {
                email,
                password_hash,
            })
            .send()
            .await;
        if let Ok(response) = response {
            (
                StatusCode::from_u16(response.status().as_u16()).expect("Should not fail"),
                axum::Json(
                    response
                        .json::<serde_json::Value>()
                        .await
                        .unwrap_or_default(),
                ),
            )
        } else {
            self.log("Auth service is not accessible", LogLevel::Error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::Value::default()),
            )
        }
    }
}
