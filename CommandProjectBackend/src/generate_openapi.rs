// Замените `crate::api::ApiDoc` на путь к вашему struct с #[derive(OpenApi)]
use crate::handlers::ApiDoc;
use utoipa::OpenApi;
use serde_json;
use std::fs;

fn main() {
    let api = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&api).expect("❌ Failed to serialize OpenAPI");
    fs::write("openapi.json", json).expect("❌ Failed to write openapi.json");
    println!("✅ OpenAPI spec generated at openapi.json");
}