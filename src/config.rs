use dotenvy::dotenv;
use std::env;
use tracing::{Level, event};

const DEFAULT_PORT: i32 = 3433;

pub struct AppConfig {
    pub database_url: String,
    pub port: i32,
}

pub fn load_config() -> Result<AppConfig, &'static str> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = match env::var("PORT") {
        Ok(port_str) => port_str.parse().unwrap_or(DEFAULT_PORT),
        Err(err) => {
            event!(Level::ERROR, "{}\nUsing default port {}", err, DEFAULT_PORT);
            DEFAULT_PORT
        }
    };

    Ok(AppConfig { database_url, port })
}
