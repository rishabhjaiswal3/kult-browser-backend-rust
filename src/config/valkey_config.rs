// src/config/valkey_config.rs
// Valkey (Redis-compatible) configuration

use std::env;

/// Valkey/Redis configuration
#[derive(Debug, Clone)]
pub struct ValkeyConfig {
    /// Valkey connection URL (required)
    pub url: String,
}

impl ValkeyConfig {
    /// Load Valkey config from environment variables.
    pub fn from_env() -> Self {
        let url = env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        Self { url }
    }
}
