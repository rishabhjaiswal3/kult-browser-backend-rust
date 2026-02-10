// src/redis/connection.rs
// Valkey/Redis connection helper

use redis::Client;

use crate::config::CONFIG;

/// Create a new Valkey/Redis client from config.
pub fn connect() -> Result<Client, String> {
    let url = &CONFIG.valkey.url;
    tracing::info!(url = %url, "Connecting to Valkey");

    Client::open(url.as_str()).map_err(|e| {
        tracing::error!(error = %e, "Failed to create Valkey client");
        format!("Valkey connection error: {}", e)
    })
}
