use dotenvy::dotenv;
use std::env;

pub struct AppConfig {
    pub database_url: String,
}

pub fn load_config() -> Result<AppConfig, &'static str> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Ok(AppConfig {
        database_url,
    })
}