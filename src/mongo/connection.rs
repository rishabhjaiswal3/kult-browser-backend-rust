use mongodb::{Client, Database};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::CONFIG;

/// Connects to MongoDB with configurable retry logic and exponential backoff.
/// Returns the Database handle on success, or an error after all retries are exhausted.
pub async fn connect() -> Result<Database, mongodb::error::Error> {
    let mongo_uri = &CONFIG.db.mongo_uri;
    let db_name = &CONFIG.db.mongo_db_name;
    let max_retries = CONFIG.db.mongo_conn_retries;

    let mut last_error = None;

    for attempt in 1..=max_retries {
        tracing::debug!(
            attempt = attempt,
            max_retries = max_retries,
            "Attempting MongoDB connection"
        );

        match Client::with_uri_str(mongo_uri).await {
            Ok(client) => {
                // Verify connection with a ping
                match client
                    .database(db_name)
                    .run_command(mongodb::bson::doc! { "ping": 1 })
                    .await
                {
                    Ok(_) => {
                        tracing::info!(db_name = %db_name, "MongoDB connected successfully");
                        return Ok(client.database(db_name));
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, attempt = attempt, "MongoDB ping failed");
                        last_error = Some(e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, attempt = attempt, "MongoDB connection failed");
                last_error = Some(e);
            }
        }

        if attempt < max_retries {
            let backoff = Duration::from_secs(2_u64.pow(attempt - 1)); // 1s, 2s, 4s...
            tracing::info!(backoff_secs = ?backoff, "Retrying MongoDB connection");
            sleep(backoff).await;
        }
    }

    tracing::error!(
        retries = max_retries,
        "All MongoDB connection attempts failed"
    );
    Err(last_error.expect("No error captured during connection attempts"))
}
