use axum::Json;
use serde_json::{json, Value};
use tracing::{event, Level};

pub async fn health() -> Json<Value> {
    const HEALTH_STATUS: bool = true;
    event!(Level::INFO, "GET - health status: {}", HEALTH_STATUS);
    Json(json!({
        "healthy": HEALTH_STATUS
    }))
}