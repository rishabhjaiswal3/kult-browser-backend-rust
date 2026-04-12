// src/config/auth_config.rs
// JWT authentication configuration

use std::env;

/// Authentication configuration for JWT tokens
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT signing secret (required in production)
    pub jwt_secret: String,
    /// Token expiration in days (default: 7)
    pub jwt_expiration_days: i64,
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
        }
    }
}
