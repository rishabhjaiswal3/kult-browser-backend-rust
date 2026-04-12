// src/moments/worker/migration_worker.rs
// Singleton MigrationWorker - consumes jobs from Valkey queue
// Downloads from DO Spaces → uploads to 0G → updates DB

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, Semaphore};
use tokio::time::timeout;

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
/// Max concurrent migration jobs. Keep this at 1 for the single-CPU DO web service.
const MAX_CONCURRENT_JOBS: usize = 1;
const IDLE_POLL_TIMEOUT_SECS: u32 = 1;
const QUEUE_ERROR_RETRY_SECS: u64 = 1;
const SHUTDOWN_DRAIN_TIMEOUT_SECS: u64 = 20;

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
            poll_timeout_secs: IDLE_POLL_TIMEOUT_SECS,
            shutdown_rx,
        }
    }

    /// Run the worker loop until a shutdown signal is received.
    /// Processes up to MAX_CONCURRENT_JOBS in parallel using a semaphore.
    pub async fn run(mut self) {
        tracing::info!(
            max_concurrent = MAX_CONCURRENT_JOBS,
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

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS));

        // Wrap shared state in Arc for spawned tasks
        let queue = Arc::new(self.queue.clone());
        let repo = Arc::new(self.repo.clone());
        let max_retries = self.max_retries;

        loop {
            tokio::select! {
                // 1. Listen for graceful shutdown
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("MigrationWorker received shutdown signal. Waiting for in-flight jobs...");
                        Self::wait_for_in_flight(semaphore.clone()).await;
                        break;
                    }
                }

                // 2. Acquire a permit, then poll the queue
                permit = semaphore.clone().acquire_owned() => {
                    let permit = match permit {
                        Ok(p) => p,
                        Err(_) => break, // Semaphore closed
                    };

                    match self.queue.pop_async::<MigrationJob>(self.poll_timeout_secs).await {
                        Ok(Some((job, raw_data))) => {
                            tracing::info!(
                                asset_id = %job.asset_id,
                                asset_type = %job.asset_type,
                                attempt = job.attempt,
                                "Processing migration job"
                            );

                            let q = queue.clone();
                            let r = repo.clone();
                            tokio::spawn(async move {
                                Self::process_job_static(&q, &r, max_retries, job).await;

                                if let Err(e) = q.ack_async(&raw_data).await {
                                    tracing::error!(error = %e, "Failed to ACK completed job");
                                }
                                drop(permit); // Release semaphore slot
                            });
                        }
                        Ok(None) => {
                            drop(permit);
                            continue; // Timeout — no jobs, loop back
                        }
                        Err(e) => {
                            drop(permit);
                            tracing::error!(error = %e, "Queue pop error, retrying shortly");
                            if !Self::sleep_or_shutdown(&mut self.shutdown_rx, Duration::from_secs(QUEUE_ERROR_RETRY_SECS)).await {
                                tracing::info!("MigrationWorker received shutdown signal during retry sleep. Waiting for in-flight jobs...");
                                Self::wait_for_in_flight(semaphore.clone()).await;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn wait_for_in_flight(semaphore: Arc<Semaphore>) {
        match timeout(
            Duration::from_secs(SHUTDOWN_DRAIN_TIMEOUT_SECS),
            semaphore.acquire_many(MAX_CONCURRENT_JOBS as u32),
        )
        .await
        {
            Ok(Ok(_permit)) => {
                tracing::info!("MigrationWorker: all in-flight jobs finished. Exiting.");
            }
            Ok(Err(_)) => {
                tracing::warn!("MigrationWorker: semaphore closed during shutdown.");
            }
            Err(_) => {
                tracing::warn!(
                    timeout_secs = SHUTDOWN_DRAIN_TIMEOUT_SECS,
                    "MigrationWorker: timed out waiting for in-flight jobs; unacked jobs will be recovered on next startup"
                );
            }
        }
    }

    async fn sleep_or_shutdown(
        shutdown_rx: &mut watch::Receiver<bool>,
        duration: Duration,
    ) -> bool {
        tokio::select! {
            _ = shutdown_rx.changed() => !*shutdown_rx.borrow(),
            _ = tokio::time::sleep(duration) => true,
        }
    }

    /// Process a single migration job (static version for spawned tasks).
    async fn process_job_static(
        queue: &ValkyQueue,
        repo: &MomentsRepository,
        max_retries: u32,
        job: MigrationJob,
    ) {
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
                Self::handle_failure_static(queue, max_retries, job).await;
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
                Self::handle_failure_static(queue, max_retries, job).await;
                return;
            }
        };

        // Step 3: Update MongoDB with 0G hash
        match repo
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
    async fn handle_failure_static(queue: &ValkyQueue, max_retries: u32, job: MigrationJob) {
        if job.attempt < max_retries {
            let retry_job = MigrationJob {
                attempt: job.attempt + 1,
                ..job
            };
            tracing::warn!(
                asset_id = %retry_job.asset_id,
                attempt = retry_job.attempt,
                "Retrying migration job"
            );
            if let Err(e) = queue.push_async(&retry_job).await {
                tracing::error!(error = %e, "Failed to re-queue job");
            }
        } else {
            let dlq_name = format!("{}:dead_letter", queue.queue_name());
            tracing::error!(
                asset_id = %job.asset_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            match ValkyQueue::new(queue.client().clone(), &dlq_name).await {
                Ok(dlq) => {
                    if let Err(e) = dlq.push_async(&job).await {
                        tracing::error!(error = %e, "Failed to push to dead letter queue");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to connect to dead letter queue");
                }
            }
        }
    }
}
