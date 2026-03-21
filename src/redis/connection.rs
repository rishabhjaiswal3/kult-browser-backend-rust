// src/redis/connection.rs
// Valkey/Redis connection helper

use std::time::Duration;

use redis::{Client, RedisResult};
use tokio::time::timeout;

use crate::config::CONFIG;

/// Create a new Valkey/Redis client from config.
pub async fn connect() -> Result<Client, String> {
    let url = &CONFIG.valkey.url;
    tracing::info!(url = %url, "Connecting to Valkey");

    let client = Client::open(url.as_str()).map_err(|e| {
        tracing::error!(error = %e, "Failed to create Valkey client");
        format!("Valkey connection error: {}", e)
    })?;

    let mut conn = timeout(
        Duration::from_secs(5),
        client.get_multiplexed_async_connection(),
    )
    .await
    .map_err(|_| "Valkey connection timed out".to_string())?
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to establish Valkey connection");
        format!("Valkey connection error: {}", e)
    })?;

    let pong: RedisResult<String> = timeout(
        Duration::from_secs(5),
        redis::cmd("PING").query_async(&mut conn),
    )
    .await
    .map_err(|_| "Valkey ping timed out".to_string())?;

    pong.map_err(|e| {
        tracing::error!(error = %e, "Failed to ping Valkey");
        format!("Valkey ping error: {}", e)
    })?;

    tracing::info!("Valkey ping successful");

    Ok(client)
}
