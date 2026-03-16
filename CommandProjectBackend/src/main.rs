use axum::http::StatusCode;
use axum::routing::get;
use dotenv;

mod handlers;

fn create_api_key_whitelist() -> impl Fn(
    axum::extract::Request,
    axum::middleware::Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<axum::response::Response, StatusCode>> + Send>> + Clone + Send + 'static {
    let real_api_key = std::env::var("API_KEY").unwrap_or_else(|_| {
        eprintln!("No API key provided in env");
        panic!();
    });
    move |request, next| {
        let real_api_key = real_api_key.clone();
        Box::pin(async move {
            let user_api_key = request
                .headers()
                .get("x-api-key")
                .and_then(|value| value.to_str().ok());
            match user_api_key {
                Some(key) => {
                    if key == real_api_key {
                        Ok(next.run(request).await)
                    } else {
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
                None => Err(StatusCode::UNAUTHORIZED),
            }
        })
    }
}

#[tokio::main]
async fn main() {
    if let Err(_) = dotenv::from_filename("server.env") {
        eprintln!("No server.env provided");
    }
    let listener = tokio::net::TcpListener::bind("localhost:8080")
        .await
        .unwrap();
    let app: axum::Router<()> = axum::Router::new()
        .route("/health_check", get(handlers::health_check))
        .layer(axum::middleware::from_fn(create_api_key_whitelist()));
    println!("Start serving");
    axum::serve(listener, app).await.unwrap();
}
