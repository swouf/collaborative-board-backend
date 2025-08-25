use axum::{Json, extract::State};
use serde_json::{Value, json};
use tokio::runtime::Handle;
use tracing::{Level, event};

use crate::AppState;

pub async fn health(
    State(AppState {
        rooms,
        db_connection_pool: _,
    }): State<AppState>,
) -> Json<Value> {
    const HEALTH_STATUS: bool = true;
    event!(Level::INFO, "GET - health status: {}", HEALTH_STATUS);

    let metrics = Handle::current().metrics();

    let alive_tasks = metrics.num_alive_tasks();

    let rooms_lock = rooms.lock().await;
    let in_memory_rooms = rooms_lock.len();

    Json(json!({
        "healthy": HEALTH_STATUS,
        "open_and_in_memory_rooms": in_memory_rooms,
        "alive_tasks": alive_tasks,
    }))
}
