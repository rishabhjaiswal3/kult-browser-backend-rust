// src/redis/queue.rs
// Reusable Valkey queue - producer/consumer pattern using async multiplexed connections

use redis::aio::MultiplexedConnection;
use redis::Client;
use serde::{de::DeserializeOwned, Serialize};

/// A reusable Valkey queue designed for async, non-blocking usage in Tokio.
///
/// Implements the Reliable Queue pattern:
/// - Uses a shared multiplexed async connection (single TCP socket, pipelined commands).
/// - `pop_async` uses `BRPOPLPUSH` to move jobs to a processing list atomically.
/// - Workers must call `ack_async` to remove the job from the processing list when done.
#[derive(Clone)]
pub struct ValkyQueue {
    client: Client,
    conn: MultiplexedConnection,
    queue_name: String,
    processing_queue_name: String,
}

impl ValkyQueue {
    /// Create a new queue instance with a shared async connection.
    pub async fn new(client: Client, queue_name: &str) -> Result<Self, String> {
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Valkey connection error: {}", e))?;
        Ok(Self {
            client,
            conn,
            queue_name: queue_name.to_string(),
            processing_queue_name: format!("{}:processing", queue_name),
        })
    }

    /// Get a reference to the underlying client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the queue name.
    pub fn queue_name(&self) -> &str {
        &self.queue_name
    }

    // ─── ASYNC METHODS ───

    /// Push a job to the queue.
    pub async fn push_async<T: Serialize + Send + 'static>(
        &self,
        payload: &T,
    ) -> Result<(), String> {
        let data = serde_json::to_string(payload).map_err(|e| format!("Serialize error: {}", e))?;
        let mut conn = self.conn.clone();

        redis::cmd("LPUSH")
            .arg(&self.queue_name)
            .arg(&data)
            .query_async::<i64>(&mut conn)
            .await
            .map_err(|e| format!("LPUSH error: {}", e))?;
        Ok(())
    }

    /// Pop a job safely using BRPOPLPUSH.
    ///
    /// Moves the job from the main queue to the processing queue atomically.
    /// Returns the typed payload AND the raw JSON string.
    /// The worker MUST pass the raw string to `ack_async` when finished.
    pub async fn pop_async<T: DeserializeOwned + Send + 'static>(
        &self,
        timeout_secs: u32,
    ) -> Result<Option<(T, String)>, String> {
        let mut conn = self.conn.clone();

        let result: Option<String> = redis::cmd("BRPOPLPUSH")
            .arg(&self.queue_name)
            .arg(&self.processing_queue_name)
            .arg(timeout_secs)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("BRPOPLPUSH error: {}", e))?;

        match result {
            Some(data) => {
                let payload: T =
                    serde_json::from_str(&data).map_err(|e| format!("Deserialize error: {}", e))?;
                Ok(Some((payload, data)))
            }
            None => Ok(None),
        }
    }

    /// Acknowledge a job is complete, removing it from the processing queue.
    pub async fn ack_async(&self, raw_data: &str) -> Result<(), String> {
        let mut conn = self.conn.clone();

        redis::cmd("LREM")
            .arg(&self.processing_queue_name)
            .arg(1)
            .arg(raw_data)
            .query_async::<i64>(&mut conn)
            .await
            .map_err(|e| format!("LREM error: {}", e))?;

        Ok(())
    }

    /// Re-queue any jobs stuck in the processing queue from a previous crash.
    pub async fn recover_stalled_jobs(&self) -> Result<u32, String> {
        let mut conn = self.conn.clone();
        let mut recovered = 0u32;

        loop {
            let result: Option<String> = redis::cmd("RPOPLPUSH")
                .arg(&self.processing_queue_name)
                .arg(&self.queue_name)
                .query_async(&mut conn)
                .await
                .map_err(|e| format!("RPOPLPUSH error: {}", e))?;

            if result.is_some() {
                recovered += 1;
            } else {
                break;
            }
        }
        Ok(recovered)
    }

    /// Get the current length of the main queue.
    pub async fn len_async(&self) -> Result<u64, String> {
        let mut conn = self.conn.clone();
        redis::cmd("LLEN")
            .arg(&self.queue_name)
            .query_async::<u64>(&mut conn)
            .await
            .map_err(|e| format!("LLEN error: {}", e))
    }

    // ─── LEGACY SYNC METHODS (For backwards-compatibility in tests/scripts) ───

    /// Push a job to the queue synchronously.
    pub fn push<T: Serialize>(&self, payload: &T) -> Result<(), String> {
        let data = serde_json::to_string(payload).map_err(|e| format!("Serialize error: {}", e))?;
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| format!("Valkey connection error: {}", e))?;

        redis::cmd("LPUSH")
            .arg(&self.queue_name)
            .arg(&data)
            .query::<i64>(&mut conn)
            .map_err(|e| format!("LPUSH error: {}", e))?;

        Ok(())
    }

    /// Pop a job from the queue synchronously (Destructive BRPOP, no processing queue).
    pub fn pop<T: DeserializeOwned>(&self, timeout_secs: u32) -> Result<Option<T>, String> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| format!("Valkey connection error: {}", e))?;

        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&self.queue_name)
            .arg(timeout_secs)
            .query(&mut conn)
            .map_err(|e| format!("BRPOP error: {}", e))?;

        match result {
            Some((_key, data)) => {
                let payload: T =
                    serde_json::from_str(&data).map_err(|e| format!("Deserialize error: {}", e))?;
                Ok(Some(payload))
            }
            None => Ok(None),
        }
    }

    /// Get the current length of the queue synchronously.
    pub fn len(&self) -> Result<u64, String> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| format!("Valkey connection error: {}", e))?;

        redis::cmd("LLEN")
            .arg(&self.queue_name)
            .query::<u64>(&mut conn)
            .map_err(|e| format!("LLEN error: {}", e))
    }
}
