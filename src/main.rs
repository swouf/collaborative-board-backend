mod ai;
mod config;
mod constants;
mod health;
mod infra;
mod models;
mod ws;

use axum::{Router, routing::get};
use config::load_config;
use deadpool_diesel::postgres::Pool;
use infra::db::setup::setup_connection_pool;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ws::{
    handler::ws_handler,
    room::{Room, Rooms},
};

use crate::{config::AppConfig, health::health};

#[derive(Clone)]
struct AppState {
    rooms: Rooms,
    db_connection_pool: Pool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let AppConfig { database_url, port } = load_config().unwrap();

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::<String, Room>::new()));
    let rooms2 = rooms.clone();

    let db_connection_pool = setup_connection_pool(database_url).await;
    let db_connection_pool2 = db_connection_pool.clone();

    let ws_router = Router::new()
        .route("/", get(ws_handler))
        .with_state(AppState {
            rooms,
            db_connection_pool,
        });

    let health_router = Router::new()
        .route("/health", get(health))
        .with_state(AppState {
            rooms: rooms2,
            db_connection_pool: db_connection_pool2,
        });

    let app = Router::new().merge(health_router).nest("/ws", ws_router);
    // run it with hyper
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
