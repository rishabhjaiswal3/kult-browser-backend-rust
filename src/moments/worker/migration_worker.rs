// src/moments/worker/migration_worker.rs
// Singleton MigrationWorker - consumes jobs from Valkey queue
// Downloads from DO Spaces → uploads to 0G → updates DB

use std::time::Duration;

use serde::{Deserialize, Serialize};

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
pub const MIGRATION_QUEUE: &str = "moments:migration";

/// Dead-letter queue for permanently failed jobs.
pub const DEAD_LETTER_QUEUE: &str = "moments:dead";

/// Singleton worker that processes migration jobs.
///
/// Only ONE instance of this should exist per process.
pub struct MigrationWorker {
    queue: ValkyQueue,
    repo: MomentsRepository,
    max_retries: u32,
    poll_timeout_secs: u32,
}

impl MigrationWorker {
    /// Create the worker (call only once).
    pub fn new(queue: ValkyQueue, repo: MomentsRepository) -> Self {
        Self {
            queue,
            repo,
            max_retries: 3,
            poll_timeout_secs: 5,
        }
    }

    /// Run the worker loop. Blocks forever, processing jobs as they come.
    pub async fn run(&self) {
        tracing::info!(
            "MigrationWorker started — listening on queue '{}'",
            MIGRATION_QUEUE
        );

        loop {
            match self.queue.pop::<MigrationJob>(self.poll_timeout_secs) {
                Ok(Some(job)) => {
                    tracing::info!(
                        asset_id = %job.asset_id,
                        asset_type = %job.asset_type,
                        attempt = job.attempt,
                        "Processing migration job"
                    );
                    self.process_job(job).await;
                }
                Ok(None) => {
                    // Timeout — no jobs, loop back and wait again
                }
                Err(e) => {
                    tracing::error!(error = %e, "Queue pop error, retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
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
            if let Err(e) = self.queue.push(&retry_job) {
                tracing::error!(error = %e, "Failed to re-queue job");
            }
        } else {
            tracing::error!(
                asset_id = %job.asset_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            // Push to dead-letter queue for manual inspection
            let dlq = ValkyQueue::new(self.queue.connection().clone(), DEAD_LETTER_QUEUE);
            if let Err(e) = dlq.push(&job) {
                tracing::error!(error = %e, "Failed to push to dead letter queue");
            }
        }
    }
}
