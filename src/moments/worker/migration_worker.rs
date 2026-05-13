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
use crate::moments::da_events::{MomentDAEventModel, MomentDAEventRepository, EVENT_ASSET_MIGRATED};
use crate::moments::model::MomentModel;
use crate::moments::repository::MomentsRepository;
use crate::onchain::{metadata_hash, ActivityType, OnchainActivityService, RecordActivityInput};
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
    /// 0G root hash from a prior successful upload.
    ///
    /// Present when a retry should only persist the hash to MongoDB instead of
    /// downloading and uploading the same asset again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_zg_hash: Option<String>,
    /// 0G transaction hash for a prior successful asset upload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_zg_tx_hash: Option<String>,
    /// 0G root hash for a prior successful metadata upload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_zg_hash: Option<String>,
    /// 0G transaction hash for a prior successful metadata upload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_zg_tx_hash: Option<String>,
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
/// - ACKs the raw payload only after success or after the failure is safely requeued/DLQ'd
/// - Listens for shutdown signal to exit gracefully
/// Max concurrent migration jobs. Keep this at 1 for the single-CPU DO web service.
const MAX_CONCURRENT_JOBS: usize = 1;
const IDLE_POLL_TIMEOUT_SECS: u32 = 1;
const QUEUE_ERROR_RETRY_SECS: u64 = 1;
const SHUTDOWN_DRAIN_TIMEOUT_SECS: u64 = 20;

#[derive(Debug)]
enum JobProcessingResult {
    Complete,
    Failed { job: MigrationJob, reason: String },
}

pub struct MigrationWorker {
    queue: ValkyQueue,
    repo: MomentsRepository,
    onchain_activity_service: Option<OnchainActivityService>,
    da_event_repo: Option<MomentDAEventRepository>,
    max_retries: u32,
    poll_timeout_secs: u32,
    shutdown_rx: watch::Receiver<bool>,
}

impl MigrationWorker {
    /// Create the worker.
    pub fn new(
        queue: ValkyQueue,
        repo: MomentsRepository,
        onchain_activity_service: Option<OnchainActivityService>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            queue,
            repo,
            onchain_activity_service,
            da_event_repo: None,
            max_retries: 3,
            poll_timeout_secs: IDLE_POLL_TIMEOUT_SECS,
            shutdown_rx,
        }
    }

    pub fn with_da_events(mut self, repo: MomentDAEventRepository) -> Self {
        self.da_event_repo = Some(repo);
        self
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
        let onchain_activity_service = Arc::new(self.onchain_activity_service.clone());
        let da_event_repo = Arc::new(self.da_event_repo.clone());
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
                            let onchain = onchain_activity_service.clone();
                            let da_repo = da_event_repo.clone();
                            tokio::spawn(async move {
                                match Self::process_job_static(&r, onchain.as_ref().as_ref(), da_repo.as_ref().as_ref(), job).await {
                                    JobProcessingResult::Complete => {
                                        if let Err(e) = q.ack_async(&raw_data).await {
                                            tracing::error!(error = %e, "Failed to ACK completed job");
                                        }
                                    }
                                    JobProcessingResult::Failed { job, reason } => {
                                        if let Err(e) = r.mark_zg_failed(&job.asset_id, &reason).await {
                                            tracing::warn!(
                                                asset_id = %job.asset_id,
                                                error = %e,
                                                "Failed to mark 0G migration failure on moment"
                                            );
                                        }
                                        tracing::error!(
                                            asset_id = %job.asset_id,
                                            attempt = job.attempt,
                                            error = %reason,
                                            "Migration job failed"
                                        );

                                        match Self::handle_failure_static(&q, max_retries, job).await {
                                            Ok(()) => {
                                                if let Err(e) = q.ack_async(&raw_data).await {
                                                    tracing::error!(error = %e, "Failed to ACK failed job after retry/DLQ");
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    error = %e,
                                                    "Failed to persist retry/DLQ job; leaving original job unacked for recovery"
                                                );
                                            }
                                        }
                                    }
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
        repo: &MomentsRepository,
        onchain_activity_service: Option<&OnchainActivityService>,
        da_event_repo: Option<&MomentDAEventRepository>,
        job: MigrationJob,
    ) -> JobProcessingResult {
        let (root_hash, asset_tx_hash) = match job
            .asset_zg_hash
            .as_deref()
            .map(str::trim)
            .filter(|hash| !hash.is_empty())
        {
            Some(root_hash) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    root_hash = %root_hash,
                    "Retrying migration with existing 0G hash"
                );
                (root_hash.to_string(), job.asset_zg_tx_hash.clone())
            }
            None => {
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
                        return JobProcessingResult::Failed {
                            job,
                            reason: format!("Failed to download from DO Spaces: {}", e),
                        };
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
                        spaces::cleanup(&download.local_path);
                        return JobProcessingResult::Failed {
                            job,
                            reason: format!("Failed to upload to 0G storage: {}", e),
                        };
                    }
                };

                spaces::cleanup(&download.local_path);
                (upload_result.root_hash, upload_result.tx_hash)
            }
        };

        let moment = match repo.find_by_moment_id(&job.asset_id).await {
            Ok(Some(moment)) => moment,
            Ok(None) => {
                return JobProcessingResult::Failed {
                    job: MigrationJob {
                        asset_zg_hash: Some(root_hash),
                        asset_zg_tx_hash: asset_tx_hash,
                        ..job
                    },
                    reason: "Moment not found in database while building 0G metadata".to_string(),
                };
            }
            Err(e) => {
                return JobProcessingResult::Failed {
                    job: MigrationJob {
                        asset_zg_hash: Some(root_hash),
                        asset_zg_tx_hash: asset_tx_hash,
                        ..job
                    },
                    reason: format!("Failed to load moment for 0G metadata: {}", e),
                };
            }
        };

        let (metadata_hash_value, metadata_tx_hash) = match job
            .metadata_zg_hash
            .as_deref()
            .map(str::trim)
            .filter(|hash| !hash.is_empty())
        {
            Some(metadata_zg_hash) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    metadata_zg_hash = %metadata_zg_hash,
                    "Retrying migration with existing 0G metadata hash"
                );
                (
                    metadata_zg_hash.to_string(),
                    job.metadata_zg_tx_hash.clone(),
                )
            }
            None => {
                let metadata = build_zg_metadata(&moment, &root_hash, asset_tx_hash.as_deref());
                match storage::upload_json_value(&format!("{}-metadata", job.asset_id), &metadata) {
                    Ok(result) => {
                        tracing::info!(
                            asset_id = %job.asset_id,
                            metadata_zg_hash = %result.root_hash,
                            "Uploaded moment metadata to 0G storage"
                        );
                        (result.root_hash, result.tx_hash)
                    }
                    Err(e) => {
                        return JobProcessingResult::Failed {
                            job: MigrationJob {
                                asset_zg_hash: Some(root_hash),
                                asset_zg_tx_hash: asset_tx_hash,
                                ..job
                            },
                            reason: format!("Failed to upload metadata to 0G storage: {}", e),
                        };
                    }
                }
            }
        };

        // Step 3: Update MongoDB with 0G hash
        match repo
            .update_zg_storage_details(
                &job.asset_id,
                &root_hash,
                asset_tx_hash.as_deref(),
                &metadata_hash_value,
                metadata_tx_hash.as_deref(),
            )
            .await
        {
            Ok(true) => {
                tracing::info!(
                    asset_id = %job.asset_id,
                    root_hash = %root_hash,
                    "Updated asset_zg_hash in database"
                );
                tracing::info!(
                    asset_id = %job.asset_id,
                    "Migration complete"
                );
                if let Some(onchain_service) = onchain_activity_service {
                    enqueue_asset_migrated_activity(
                        onchain_service,
                        &moment,
                        &root_hash,
                        asset_tx_hash.as_deref(),
                        &metadata_hash_value,
                        metadata_tx_hash.as_deref(),
                    )
                    .await;
                }

                // Record the migration on 0G DA — proves asset is permanently stored
                if let Some(da_repo) = da_event_repo {
                    let event = MomentDAEventModel::new(
                        &moment.moment_id,
                        EVENT_ASSET_MIGRATED,
                        &moment.player_wallet_address,
                        serde_json::json!({
                            "assetZgHash": &root_hash,
                            "assetZgTxHash": asset_tx_hash.as_deref(),
                            "metadataZgHash": &metadata_hash_value,
                            "metadataZgTxHash": metadata_tx_hash.as_deref(),
                        }),
                    );
                    if let Err(e) = da_repo.create(event).await {
                        tracing::warn!(
                            moment_id = %moment.moment_id,
                            error = %e,
                            "Failed to enqueue asset_migrated DA event"
                        );
                    }
                }

                JobProcessingResult::Complete
            }
            Ok(false) => JobProcessingResult::Failed {
                job: MigrationJob {
                    asset_zg_hash: Some(root_hash),
                    asset_zg_tx_hash: asset_tx_hash,
                    metadata_zg_hash: Some(metadata_hash_value),
                    metadata_zg_tx_hash: metadata_tx_hash,
                    ..job
                },
                reason: "Moment not found in database while saving 0G hash".to_string(),
            },
            Err(e) => JobProcessingResult::Failed {
                job: MigrationJob {
                    asset_zg_hash: Some(root_hash),
                    asset_zg_tx_hash: asset_tx_hash,
                    metadata_zg_hash: Some(metadata_hash_value),
                    metadata_zg_tx_hash: metadata_tx_hash,
                    ..job
                },
                reason: format!("Failed to update database with 0G hash: {}", e),
            },
        }
    }

    /// Handle a failed job: retry or send to dead-letter queue.
    async fn handle_failure_static(
        queue: &ValkyQueue,
        max_retries: u32,
        job: MigrationJob,
    ) -> Result<(), String> {
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
            queue
                .push_async(&retry_job)
                .await
                .map_err(|e| format!("Failed to re-queue job: {}", e))?;
        } else {
            tracing::error!(
                asset_id = %job.asset_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            let dlq = ValkyQueue::new(queue.client().clone(), DEAD_LETTER_QUEUE.as_str())
                .await
                .map_err(|e| format!("Failed to connect to dead letter queue: {}", e))?;

            dlq.push_async(&job)
                .await
                .map_err(|e| format!("Failed to push to dead letter queue: {}", e))?;
        }

        Ok(())
    }
}

fn build_zg_metadata(
    moment: &MomentModel,
    asset_zg_hash: &str,
    asset_zg_tx_hash: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "kult.moment.v1",
        "momentId": &moment.moment_id,
        "title": &moment.title,
        "description": &moment.description,
        "tags": &moment.tags,
        "relatedGames": &moment.related_games,
        "creatorWallet": &moment.player_wallet_address,
        "assetUrl": &moment.asset_url,
        "assetZgHash": asset_zg_hash,
        "assetZgTxHash": asset_zg_tx_hash,
        "assetMetadata": moment.asset_metadata.as_ref().and_then(|doc| serde_json::to_value(doc).ok()),
        "socialMediaLinks": moment.social_media_links.as_ref().and_then(|doc| serde_json::to_value(doc).ok()),
        "createdAt": moment.created_at.and_then(|dt| dt.try_to_rfc3339_string().ok()),
        "updatedAt": moment.updated_at.and_then(|dt| dt.try_to_rfc3339_string().ok()),
    })
}

async fn enqueue_asset_migrated_activity(
    onchain_service: &OnchainActivityService,
    moment: &MomentModel,
    asset_zg_hash: &str,
    asset_zg_tx_hash: Option<&str>,
    metadata_zg_hash: &str,
    metadata_zg_tx_hash: Option<&str>,
) {
    let metadata = serde_json::json!({
        "assetZgHash": asset_zg_hash,
        "assetZgTxHash": asset_zg_tx_hash,
        "metadataZgHash": metadata_zg_hash,
        "metadataZgTxHash": metadata_zg_tx_hash,
    });

    if let Err(e) = onchain_service
        .enqueue_activity(RecordActivityInput {
            user_wallet: moment.player_wallet_address.clone(),
            activity_type: ActivityType::AssetMigratedTo0g,
            moment_id: moment.moment_id.clone(),
            entity_id: metadata_zg_hash.to_string(),
            metadata_hash: metadata_hash(&metadata),
        })
        .await
    {
        tracing::warn!(
            moment_id = %moment.moment_id,
            error = %e,
            "Failed to enqueue 0G asset migration on-chain activity"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::MigrationJob;

    #[test]
    fn migration_job_deserializes_legacy_payload_without_asset_zg_hash() {
        let payload = r#"{
            "assetUrl": "https://example.com/moment.jpg",
            "assetId": "moment-1",
            "assetType": "image/jpeg",
            "attempt": 1
        }"#;

        let job: MigrationJob = serde_json::from_str(payload).expect("valid legacy payload");

        assert_eq!(job.asset_id, "moment-1");
        assert_eq!(job.asset_zg_hash, None);
    }

    #[test]
    fn migration_job_serializes_asset_zg_hash_for_db_retry() {
        let job = MigrationJob {
            asset_url: "https://example.com/moment.jpg".to_string(),
            asset_id: "moment-1".to_string(),
            asset_type: "image/jpeg".to_string(),
            asset_zg_hash: Some("0xroot".to_string()),
            asset_zg_tx_hash: Some("0xtx".to_string()),
            metadata_zg_hash: Some("0xmetadata".to_string()),
            metadata_zg_tx_hash: Some("0xmetatx".to_string()),
            attempt: 2,
        };

        let value = serde_json::to_value(job).expect("serializable job");

        assert_eq!(value["assetZgHash"], "0xroot");
        assert_eq!(value["assetZgTxHash"], "0xtx");
        assert_eq!(value["metadataZgHash"], "0xmetadata");
        assert_eq!(value["metadataZgTxHash"], "0xmetatx");
        assert_eq!(value["attempt"], 2);
    }

    #[test]
    fn migration_job_omits_asset_zg_hash_for_fresh_jobs() {
        let job = MigrationJob {
            asset_url: "https://example.com/moment.jpg".to_string(),
            asset_id: "moment-1".to_string(),
            asset_type: "image/jpeg".to_string(),
            asset_zg_hash: None,
            asset_zg_tx_hash: None,
            metadata_zg_hash: None,
            metadata_zg_tx_hash: None,
            attempt: 1,
        };

        let value = serde_json::to_value(job).expect("serializable job");

        assert!(value.get("assetZgHash").is_none());
        assert!(value.get("assetZgTxHash").is_none());
        assert!(value.get("metadataZgHash").is_none());
        assert!(value.get("metadataZgTxHash").is_none());
    }
}
