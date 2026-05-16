// src/moments/worker/da_event_worker.rs
//
// Real 0G DA pipeline — NOT 0G Storage.
//
// Two-phase workflow per cycle:
//   Phase 1 (disperse): pick up "pending" events → POST blob to 0G DA disperser → mark "dispersing"
//   Phase 2 (finalize): pick up "dispersing" events → GET blob status → mark "finalized" with receipt
//
// DA blob schema: kult.event.v1 — a canonical JSON envelope containing event metadata.
// The batchHeaderHash from the finalized receipt is the on-chain proof anchor.

use std::time::Duration;
use tokio::sync::watch;

use crate::external::da::{DaBlobStatus, ZgDaClient};
use crate::moments::da_events::{MomentDAEventModel, MomentDAEventRepository};

const POLL_INTERVAL_SECS: u64 = 15;
const BATCH_SIZE: i64 = 10;

pub struct DAEventWorker {
    repo: MomentDAEventRepository,
    da_client: ZgDaClient,
    shutdown_rx: watch::Receiver<bool>,
}

impl DAEventWorker {
    pub fn new(
        repo: MomentDAEventRepository,
        da_client: ZgDaClient,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            repo,
            da_client,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        tracing::info!("DAEventWorker started — real 0G DA disperser pipeline active");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("DAEventWorker received shutdown signal");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)) => {
                    if let Err(e) = self.process_cycle().await {
                        tracing::error!(error = %e, "DAEventWorker cycle failed");
                    }
                }
            }
        }
    }

    async fn process_cycle(&self) -> Result<(), String> {
        self.disperse_pending().await?;
        self.poll_finalization().await?;
        Ok(())
    }

    /// Phase 1: Submit pending events to the 0G DA disperser.
    async fn disperse_pending(&self) -> Result<(), String> {
        let events = self.repo.find_pending(BATCH_SIZE).await?;

        for event in events {
            let event_id = match event.id {
                Some(ref id) => id.clone(),
                None => continue,
            };

            let blob = Self::build_blob(&event);
            let blob_bytes = match serde_json::to_vec(&blob) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to serialize DA blob");
                    continue;
                }
            };

            match self.da_client.disperse_blob(&blob_bytes).await {
                Ok(request_id) => {
                    self.repo.mark_dispersing(&event_id, &request_id).await?;
                    tracing::info!(
                        event_type = %event.event_type,
                        moment_id = %event.moment_id,
                        request_id = %request_id,
                        "DA blob submitted to 0G disperser"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        event_type = %event.event_type,
                        moment_id = %event.moment_id,
                        error = %e,
                        "Failed to submit DA blob to 0G disperser"
                    );
                    self.repo.mark_failed(&event_id, &e).await?;
                }
            }
        }

        Ok(())
    }

    /// Phase 2: Poll disperser for finalization of previously submitted blobs.
    async fn poll_finalization(&self) -> Result<(), String> {
        let events = self.repo.find_dispersing(BATCH_SIZE).await?;

        for event in events {
            let event_id = match event.id {
                Some(ref id) => id.clone(),
                None => continue,
            };

            let request_id = match event.da_request_id.as_deref() {
                Some(id) => id.to_string(),
                None => continue,
            };

            match self.da_client.get_blob_status(&request_id).await {
                Ok(DaBlobStatus::Finalized(receipt)) => {
                    tracing::info!(
                        event_type = %event.event_type,
                        moment_id = %event.moment_id,
                        batch_id = ?receipt.batch_id,
                        blob_index = ?receipt.blob_index,
                        batch_header_hash = ?receipt.batch_header_hash,
                        "DA blob finalized on 0G"
                    );
                    self.repo.mark_finalized(&event_id, &receipt).await?;
                }
                Ok(DaBlobStatus::Failed(e)) => {
                    tracing::warn!(
                        event_type = %event.event_type,
                        moment_id = %event.moment_id,
                        error = %e,
                        "DA blob finalization failed"
                    );
                    self.repo.mark_failed(&event_id, &e).await?;
                }
                Ok(DaBlobStatus::Processing)
                | Ok(DaBlobStatus::Confirmed)
                | Ok(DaBlobStatus::Unknown) => {
                    // Still in flight — check next cycle
                }
                Err(e) => {
                    tracing::warn!(
                        request_id = %request_id,
                        error = %e,
                        "Failed to poll DA blob status"
                    );
                }
            }
        }

        Ok(())
    }

    /// Build the canonical DA blob envelope for a moment event.
    /// Schema version is included so future parsers can handle format changes.
    fn build_blob(event: &MomentDAEventModel) -> serde_json::Value {
        serde_json::json!({
            "schema": "kult.event.v1",
            "eventType": event.event_type,
            "momentId": event.moment_id,
            "actorWallet": event.actor_wallet,
            "timestamp": event.created_at.and_then(|dt| dt.try_to_rfc3339_string().ok()),
            "data": serde_json::to_value(&event.event_data).unwrap_or(serde_json::Value::Null),
        })
    }
}
