// src/config/mod.rs
// Central configuration module - loads all config from environment

pub mod app_config;
pub mod db_config;
pub mod log_config;

use once_cell::sync::Lazy;

pub use app_config::AppConfig;
pub use db_config::DbConfig;
pub use log_config::LogConfig;

/// Global configuration loaded once at startup
#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
    pub log: LogConfig,
    // Future: pub auth: AuthConfig,
    // Future: pub redis: RedisConfig,
}

impl Config {
    /// Load all configuration from environment.
    /// Call this after dotenvy::dotenv() has loaded .env
    fn from_env() -> Self {
        Self {
            app: AppConfig::from_env(),
            db: DbConfig::from_env(),
            log: LogConfig::from_env(),
        }
    }
}

/// Global config instance - access via `CONFIG.app.port`, `CONFIG.db.mongo_uri`, `CONFIG.log.level`, etc.
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // Load .env file (ignore if missing - production uses real env vars)
    let _ = dotenvy::dotenv();
    Config::from_env()
});
