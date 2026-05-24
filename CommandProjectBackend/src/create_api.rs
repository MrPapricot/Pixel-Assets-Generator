use std::fs;
mod handlers;
use utoipa::OpenApi;
mod app_state;
mod rpc_implementation;

fn main() {
    let api = handlers::ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&api).expect("❌ Failed to serialize OpenAPI");
    fs::write("openapi.json", json).expect("❌ Failed to write openapi.json");
    println!("✅ OpenAPI spec generated at openapi.json");
}
