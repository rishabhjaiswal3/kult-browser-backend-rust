// src/referral/worker.rs

use crate::redis::ValkyQueue;
use crate::referral::anti_fraud::PendingReferralCheck;
use crate::referral::model::ReferralConnection;
use crate::referral::service::ReferralService;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Database;
use std::sync::Arc;
use tokio::sync::watch;

pub struct EvaluationWorker {
    verify_queue: ValkyQueue,
    referral_service: Arc<ReferralService>,
    db: Database,
}

impl EvaluationWorker {
    pub fn new(
        verify_queue: ValkyQueue,
        referral_service: Arc<ReferralService>,
        db: Database,
    ) -> Self {
        Self {
            verify_queue,
            referral_service,
            db,
        }
    }

    pub async fn run(self, mut shutdown_rx: watch::Receiver<bool>) {
        tracing::info!("EvaluationWorker for referrals started");

        let connections_coll = self.db.collection::<ReferralConnection>("store_referrals");
        let scores_coll = self
            .db
            .collection::<mongodb::bson::Document>("store_referral_scores");

        loop {
            tokio::select! {
                // Wait for graceful shutdown signal
                _ = shutdown_rx.changed() => {
                    tracing::info!("EvaluationWorker received shutdown signal. Exiting...");
                    break;
                }

                // Poll the queue (blocks for up to 5 seconds per iteration)
                job_result = self.verify_queue.pop_async::<PendingReferralCheck>(5) => {
                    match job_result {
                        Ok(Some((pending_check, raw_job_data))) => {
                            tracing::info!(
                                referred = %pending_check.new_player_id,
                                code = %pending_check.referred_by_code,
                                "Evaluating referral connection"
                            );

                            // 1. Resolve referrer wallet
                            let referrer_wallet = match self
                                .referral_service
                                .resolve_code_to_wallet(&pending_check.referred_by_code)
                                .await
                            {
                                Ok(Some(w)) => w,
                                Ok(None) => {
                                    tracing::warn!("Referral code not found: {}", pending_check.referred_by_code);
                                    let _ = self.verify_queue.ack_async(&raw_job_data).await;
                                    continue;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to resolve code: {}", e);
                                    continue; // Retry later
                                }
                            };

                            // Prevent self-referrals
                            if referrer_wallet == pending_check.new_player_id {
                                tracing::warn!("Blocked self-referral: {}", referrer_wallet);
                                let _ = self.verify_queue.ack_async(&raw_job_data).await;
                                continue;
                            }

                            // 2. Double-check MongoDB to ensure this user hasn't already been referred
                            let existing = connections_coll
                                .find_one(doc! { "referred_player_id": &pending_check.new_player_id })
                                .await
                                .unwrap_or(None);

                            if existing.is_some() {
                                tracing::warn!("User already referred: {}", pending_check.new_player_id);
                                let _ = self.verify_queue.ack_async(&raw_job_data).await;
                                continue;
                            }

                            // 3. Create historical Permanent Record
                            let connection = ReferralConnection {
                                id: None,
                                referrer_id: referrer_wallet.clone(),
                                referred_player_id: pending_check.new_player_id.clone(),
                                code_used: pending_check.referred_by_code.clone(),
                                reward_issued: 100, // Hardcoded 100 points for now
                                created_at: Utc::now(),
                            };

                            if let Err(e) = connections_coll.insert_one(&connection).await {
                                tracing::error!("Failed to save ReferralConnection: {}", e);
                                continue;
                            }

                            // 4. Update the Referrer's Score in a Game-Leaderboard-compatible collection
                            // This allows the GlobalLeaderboardService to pick it up later
                            let options = mongodb::options::UpdateOptions::builder()
                                .upsert(true)
                                .build();
                            if let Err(e) = scores_coll
                                .update_one(
                                    doc! { "walletAddress": &referrer_wallet },
                                    doc! {
                                        "$inc": { "score": 100 },
                                        "$setOnInsert": { "walletAddress": &referrer_wallet }
                                    },
                                )
                                .with_options(options)
                                .await
                            {
                                tracing::error!("Failed to increment referral score: {}", e);
                            }

                            // 5. Acknowledge and remove the job from Valkey
                            if let Err(e) = self.verify_queue.ack_async(&raw_job_data).await {
                                tracing::error!("Failed to ack referral verify job: {}", e);
                            } else {
                                tracing::info!("Referral successfully rewarded for {}", referrer_wallet);
                            }
                        }
                        Ok(None) => {
                            // Queue empty, just loop again (BRPOPLPUSH handled the 5s wait)
                        }
                        Err(e) => {
                            tracing::error!("Error polling verify queue: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
    }
}
