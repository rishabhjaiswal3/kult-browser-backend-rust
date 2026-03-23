// src/moments/worker/migration_worker.rs
// Singleton MigrationWorker - consumes jobs from Valkey queue
// Downloads from DO Spaces → uploads to 0G → updates DB

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::watch;

use crate::external::spaces;
use crate::external::storage;
use crate::moments::repository::MomentsRepository;
use crate::redis::ValkyQueue;

/// Job payload pushed by the service, popped by the worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationJob {
    /// DigitalOcean Spaces URL to download from
    pub asset_url: String,
    /// NanoID of the moment (= moment_id)
    pub asset_id: String,
    /// MIME type (e.g. "image/gif")
    pub asset_type: String,
    /// Retry attempt number
    pub attempt: u32,
}

/// Queue name used for moment migration jobs.
pub use crate::redis::keys::MOMENTS_ZERO_G_MIGRATION_QUEUE as MIGRATION_QUEUE;

/// Dead-letter queue for permanently failed jobs.
pub use crate::redis::keys::MOMENTS_ZERO_G_MIGRATION_DEAD_LETTER_QUEUE as DEAD_LETTER_QUEUE;

/// Singleton worker that processes migration jobs.
///
/// Converts to Reliable Queue pattern:
/// - Polls queue via `pop_async` (BRPOPLPUSH)
/// - Process: download, upload to 0G, update MongoDB, cleanup
/// - Unconditionally `ack_async`s the raw payload when done (even on retry/DLQ)
/// - Listens for shutdown signal to exit gracefully
pub struct MigrationWorker {
    queue: ValkyQueue,
    repo: MomentsRepository,
    max_retries: u32,
    poll_timeout_secs: u32,
    shutdown_rx: watch::Receiver<bool>,
}

impl MigrationWorker {
    /// Create the worker.
    pub fn new(
        queue: ValkyQueue,
        repo: MomentsRepository,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            queue,
            repo,
            max_retries: 3,
            poll_timeout_secs: 5,
            shutdown_rx,
        }
    }

    /// Run the worker loop until a shutdown signal is received.
    pub async fn run(mut self) {
        tracing::info!(
            "MigrationWorker started — listening on queue '{}'",
            self.queue.queue_name()
        );

        // On startup, we must recover any jobs that were stuck in the processing queue
        // due to a previous crash.
        match self.queue.recover_stalled_jobs().await {
            Ok(count) if count > 0 => {
                tracing::info!("Recovered {} stalled jobs from processing queue", count);
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "Failed to recover stalled jobs"),
        }

        loop {
            tokio::select! {
                // 1. Listen for graceful shutdown
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("MigrationWorker received shutdown signal. Exiting gracefully.");
                        break;
                    }
                }

                // 2. Poll the queue
                pop_result = self.queue.pop_async::<MigrationJob>(self.poll_timeout_secs) => {
                    match pop_result {
                        Ok(Some((job, raw_data))) => {
                            tracing::info!(
                                asset_id = %job.asset_id,
                                asset_type = %job.asset_type,
                                attempt = job.attempt,
                                "Processing migration job"
                            );

                            self.process_job(job).await;

                            // UNCONDITIONAL ACK: Whether successful, retried, or DLQ'd, the original popped job is done.
                            if let Err(e) = self.queue.ack_async(&raw_data).await {
                                tracing::error!(error = %e, "Failed to ACK completed job");
                            }
                        }
                        Ok(None) => continue, // Timeout — no jobs, loop back
                        Err(e) => {
                            tracing::error!(error = %e, "Queue pop error, retrying in 5s");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    }

    /// Process a single migration job:
    /// 1. Download file from DO Spaces
    /// 2. Upload to 0G storage
    /// 3. Update asset_zg_hash in MongoDB
    /// 4. Cleanup temp file
    async fn process_job(&self, job: MigrationJob) {
        // Step 1: Download from DO Spaces
        let download = match spaces::download_file(&job.asset_url).await {
            Ok(result) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    path = %result.local_path.display(),
                    size = result.size_bytes,
                    "Downloaded from DO Spaces"
                );
                result
            }
            Err(e) => {
                tracing::error!(
                    asset_id = %job.asset_id,
                    error = %e,
                    "Failed to download from DO Spaces"
                );
                self.handle_failure(job).await;
                return;
            }
        };

        // Step 2: Upload to 0G storage
        let local_path_str = download.local_path.to_string_lossy().to_string();
        let upload_result = match storage::upload_file(&local_path_str) {
            Ok(result) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    root_hash = %result.root_hash,
                    "Uploaded to 0G storage"
                );
                result
            }
            Err(e) => {
                tracing::error!(
                    asset_id = %job.asset_id,
                    error = %e,
                    "Failed to upload to 0G storage"
                );
                // Cleanup temp file before retrying
                spaces::cleanup(&download.local_path);
                self.handle_failure(job).await;
                return;
            }
        };

        // Step 3: Update MongoDB with 0G hash
        match self
            .repo
            .update_zg_hash(&job.asset_id, &upload_result.root_hash)
            .await
        {
            Ok(true) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    root_hash = %upload_result.root_hash,
                    "Updated asset_zg_hash in database"
                );
            }
            Ok(false) => {
                tracing::warn!(
                    asset_id = %job.asset_id,
                    "Moment not found in database — hash not updated"
                );
            }
            Err(e) => {
                tracing::error!(
                    asset_id = %job.asset_id,
                    error = %e,
                    "Failed to update database — 0G hash may be lost"
                );
            }
        }

        // Step 4: Cleanup temp file (always, even if DB update failed)
        spaces::cleanup(&download.local_path);

        tracing::info!(
            asset_id = %job.asset_id,
            "Migration complete"
        );
    }

    /// Handle a failed job: retry or send to dead-letter queue.
    async fn handle_failure(&self, job: MigrationJob) {
        if job.attempt < self.max_retries {
            let retry_job = MigrationJob {
                attempt: job.attempt + 1,
                ..job
            };
            tracing::warn!(
                asset_id = %retry_job.asset_id,
                attempt = retry_job.attempt,
                "Retrying migration job"
            );
            if let Err(e) = self.queue.push_async(&retry_job).await {
                tracing::error!(error = %e, "Failed to re-queue job");
            }
        } else {
            let dlq_name = format!("{}:dead_letter", self.queue.queue_name());
            tracing::error!(
                asset_id = %job.asset_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            // Push to dead-letter queue for manual inspection
            let dlq = ValkyQueue::new(self.queue.connection().clone(), &dlq_name);
            if let Err(e) = dlq.push_async(&job).await {
                tracing::error!(error = %e, "Failed to push to dead letter queue");
            }
        }
    }
}
