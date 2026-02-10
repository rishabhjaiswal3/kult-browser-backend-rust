// src/redis/queue.rs
// Reusable Valkey queue - producer/consumer pattern

use redis::{Client, Connection};
use serde::{de::DeserializeOwned, Serialize};

/// A simple, reusable Valkey queue.
///
/// Works with any serializable payload type.
/// Uses LPUSH/BRPOP for FIFO ordering.
#[derive(Clone)]
pub struct ValkyQueue {
    client: Client,
    queue_name: String,
}

impl ValkyQueue {
    /// Create a new queue instance.
    pub fn new(client: Client, queue_name: &str) -> Self {
        Self {
            client,
            queue_name: queue_name.to_string(),
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

    /// Push a job to the queue (producer).
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

    /// Pop a job from the queue (consumer).
    /// Blocks until a job is available or timeout (seconds) is reached.
    /// timeout = 0 means block forever.
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
            None => Ok(None), // Timeout, no job available
        }
    }

    /// Get the current length of the queue.
    pub fn len(&self) -> Result<u64, String> {
        let mut conn = self.conn()?;

        redis::cmd("LLEN")
            .arg(&self.queue_name)
            .query::<u64>(&mut conn)
            .map_err(|e| format!("LLEN error: {}", e))
    }
}
