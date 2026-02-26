use axum::routing::get;

mod handlers;
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("localhost:8080").await.unwrap();
    let app: axum::Router<()> = axum::Router::new()
        .route("/health_check", get(handlers::health_check));
    axum::serve(listener, app).await.unwrap();
}
