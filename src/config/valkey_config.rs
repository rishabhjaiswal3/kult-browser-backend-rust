// src/config/valkey_config.rs
// Valkey (Redis-compatible) configuration

use std::env;

/// Valkey/Redis configuration
#[derive(Debug, Clone)]
pub struct ValkeyConfig {
    /// Valkey connection URL (required)
    pub url: String,
    /// Prefix applied to backend-managed Redis keys
    pub key_prefix: String,
}

impl ValkeyConfig {
    /// Load Valkey config from environment variables.
    pub fn from_env() -> Self {
        let url = env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let key_prefix = env::var("VALKEY_KEY_PREFIX")
            .unwrap_or_else(|_| "kult_browser_rust".to_string())
            .trim()
            .trim_matches(':')
            .to_string();

        Self {
            url,
            key_prefix: if key_prefix.is_empty() {
                "kult_browser_rust".to_string()
            } else {
                key_prefix
            },
        }
    }
}
