// src/config/app_config.rs
// Application-level configuration (server, CORS, logging)

use std::env;

/// Application configuration for server settings
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Server port (default: 3000)
    pub port: u16,
    /// Server bind address (default: 0.0.0.0)
    pub host: String,
    /// Application name (default: kult-browser-backend)
    pub app_name: String,
    /// Runtime environment. Admin routes are enabled only when this is "dev".
    pub environment: String,
    /// CORS allowed origins (default: *)
    pub cors_origins: Vec<String>,
    /// Log level (default: info)
    pub log_level: String,
}

impl AppConfig {
    /// Load app config from environment variables.
    /// Panics with clear error message if required vars are missing.
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid u16"),

            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            app_name: env::var("APP_NAME").unwrap_or_else(|_| "kult-browser-backend".to_string()),

            environment: env::var("ENVIRONMENT")
                .or_else(|_| env::var("APP_ENV"))
                .unwrap_or_else(|_| "prod".to_string())
                .trim()
                .to_ascii_lowercase(),

            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn admin_routes_enabled(&self) -> bool {
        self.environment == "dev"
    }
}
