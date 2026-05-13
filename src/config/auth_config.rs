// src/config/auth_config.rs
// JWT authentication configuration

use std::env;

/// Authentication configuration for JWT tokens and SIWE
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT signing secret (required in production)
    pub jwt_secret: String,
    /// Token expiration in days (default: 7)
    pub jwt_expiration_days: i64,
    /// Domain shown in SIWE message (default: app.kultgames.io)
    pub siwe_domain: String,
    /// URI shown in SIWE message (default: https://app.kultgames.io)
    pub siwe_uri: String,
    /// EVM chain ID for SIWE (default: 1 = Ethereum mainnet)
    pub siwe_chain_id: u64,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me".to_string()),
            jwt_expiration_days: env::var("JWT_EXPIRATION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            siwe_domain: env::var("SIWE_DOMAIN")
                .unwrap_or_else(|_| "app.kultgames.io".to_string()),
            siwe_uri: env::var("SIWE_URI")
                .unwrap_or_else(|_| "https://app.kultgames.io".to_string()),
            siwe_chain_id: env::var("SIWE_CHAIN_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        }
    }
}
