use logger::{LogLevel, Logger};
use reqwest;
use std::sync::{Arc, Mutex, RwLock};

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
    #[serde(rename="Name")]
    pub(crate) service_name: String,
    #[serde(rename="Status")]
    pub(crate) service_status: Status,
    #[serde(rename="JSONMessage")]
    pub(crate) service_json_output: Option<serde_json::Value>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    logger: Arc<Mutex<dyn Logger>>,
    services: Arc<RwLock<Vec<ServiceData>>>,
}

impl AppState {
    pub fn new(
        logger: Arc<Mutex<dyn Logger>>,
        services: Arc<RwLock<Vec<ServiceData>>>,
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

    pub async fn check_services(self) -> Vec<ServiceStatus> {
        // TODO Переписать все на ureq вместо reqwest
        let services: Vec<ServiceData> = (*self.services.read().expect("Poisoned, Should not happen")).clone();
        let mut responses = tokio::task::JoinSet::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Should not fail");
        for service in services {
            let client = client.clone();
            responses.spawn(async move {
                // TODO Сделать нормальную обработку ошибок
                let response = client
                    .get(format!(
                        "http://{}:{}/health",
                        service.service_host.as_str(),
                        service.service_port
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
}
