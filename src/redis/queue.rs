// src/redis/queue.rs
// Reusable Valkey queue - producer/consumer pattern

use redis::{Client, Connection};
use serde::{de::DeserializeOwned, Serialize};

/// A reusable Valkey queue designed for async, non-blocking usage in Tokio.
///
/// Implements the Reliable Queue pattern:
/// - `pop_async` uses `BRPOPLPUSH` to move jobs to a processing list atomically.
/// - Workers must call `ack_async` to remove the job from the processing list when done.
/// - All Redis operations run in `tokio::task::spawn_blocking` to prevent Tokio thread starvation.
#[derive(Clone)]
pub struct ValkyQueue {
    client: Client,
    queue_name: String,
    processing_queue_name: String,
}

impl ValkyQueue {
    /// Create a new queue instance.
    pub fn new(client: Client, queue_name: &str) -> Self {
        Self {
            client,
            queue_name: queue_name.to_string(),
            processing_queue_name: format!("{}_processing", queue_name),
        }
    }

    /// Get a connection from the client.
    fn conn(&self) -> Result<Connection, String> {
        self.client
            .get_connection()
            .map_err(|e| format!("Valkey connection error: {}", e))
    }

    /// Get a reference to the underlying client.
    pub fn connection(&self) -> &Client {
        &self.client
    }

    // ─── ASYNC METHODS (Non-blocking, use these in Tokio context) ───

    /// Push a job to the queue asynchronously.
    pub async fn push_async<T: Serialize + Send + 'static>(
        &self,
        payload: &T,
    ) -> Result<(), String> {
        let data = serde_json::to_string(payload).map_err(|e| format!("Serialize error: {}", e))?;
        let queue = self.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = queue.conn()?;
            redis::cmd("LPUSH")
                .arg(&queue.queue_name)
                .arg(&data)
                .query::<i64>(&mut conn)
                .map_err(|e| format!("LPUSH error: {}", e))?;
            Ok(())
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
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
        let queue = self.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = queue.conn()?;

            // BRPOPLPUSH src dst timeout
            // Returns the popped element, or Nil if timeout
            let result: Option<String> = redis::cmd("BRPOPLPUSH")
                .arg(&queue.queue_name)
                .arg(&queue.processing_queue_name)
                .arg(timeout_secs)
                .query(&mut conn)
                .map_err(|e| format!("BRPOPLPUSH error: {}", e))?;

            match result {
                Some(data) => {
                    let payload: T = serde_json::from_str(&data)
                        .map_err(|e| format!("Deserialize error: {}", e))?;
                    Ok(Some((payload, data)))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Acknowledge a job is complete, removing it from the processing queue.
    ///
    /// The worker must pass the raw string returned from `pop_async`.
    pub async fn ack_async(&self, raw_data: &str) -> Result<(), String> {
        let queue = self.clone();
        let data = raw_data.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = queue.conn()?;

            // LREM key count value
            // count > 0: Remove elements equal to value moving from head to tail
            redis::cmd("LREM")
                .arg(&queue.processing_queue_name)
                .arg(1)
                .arg(&data)
                .query::<i64>(&mut conn)
                .map_err(|e| format!("LREM error: {}", e))?;

            Ok(())
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Re-queue any jobs stuck in the processing queue from a previous crash.
    ///
    /// Runs RPOPLPUSH repeatedly until the processing queue is empty.
    /// Call this once on application startup for each queue.
    pub async fn recover_stalled_jobs(&self) -> Result<u32, String> {
        let queue = self.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = queue.conn()?;
            let mut recovered = 0;

            loop {
                // RPOPLPUSH moves rightmost element to left of destination
                let result: Option<String> = redis::cmd("RPOPLPUSH")
                    .arg(&queue.processing_queue_name)
                    .arg(&queue.queue_name)
                    .query(&mut conn)
                    .map_err(|e| format!("RPOPLPUSH error: {}", e))?;

                if result.is_some() {
                    recovered += 1;
                } else {
                    break;
                }
            }
            Ok(recovered)
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    /// Get the current length of the main queue asynchronously.
    pub async fn len_async(&self) -> Result<u64, String> {
        let queue = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = queue.conn()?;
            redis::cmd("LLEN")
                .arg(&queue.queue_name)
                .query::<u64>(&mut conn)
                .map_err(|e| format!("LLEN error: {}", e))
        })
        .await
        .map_err(|e| format!("Spawn blocking error: {}", e))?
    }

    // ─── LEGACY SYNC METHODS (For backwards-compatibility in tests/scripts) ───

    /// Push a job to the queue synchronously.
    pub fn push<T: Serialize>(&self, payload: &T) -> Result<(), String> {
        let data = serde_json::to_string(payload).map_err(|e| format!("Serialize error: {}", e))?;
        let mut conn = self.conn()?;

        redis::cmd("LPUSH")
            .arg(&self.queue_name)
            .arg(&data)
            .query::<i64>(&mut conn)
            .map_err(|e| format!("LPUSH error: {}", e))?;

        Ok(())
    }

    /// Pop a job from the queue synchronously (Destructive BRPOP, no processing queue).
    pub fn pop<T: DeserializeOwned>(&self, timeout_secs: u32) -> Result<Option<T>, String> {
        let mut conn = self.conn()?;

        // BRPOP returns Option<(queue_name, value)>
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
        let mut conn = self.conn()?;

        redis::cmd("LLEN")
            .arg(&self.queue_name)
            .query::<u64>(&mut conn)
            .map_err(|e| format!("LLEN error: {}", e))
    }
}
