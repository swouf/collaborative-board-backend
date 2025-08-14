use axum::Json;
use serde_json::{Value, json};
use tracing::{Level, event};

pub async fn health() -> Json<Value> {
    const HEALTH_STATUS: bool = true;
    event!(Level::INFO, "GET - health status: {}", HEALTH_STATUS);
    Json(json!({
        "healthy": HEALTH_STATUS
    }))
}
