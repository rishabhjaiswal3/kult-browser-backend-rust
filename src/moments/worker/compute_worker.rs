// src/moments/worker/compute_worker.rs
// Polls moments with zgStatus=stored and no aiStatus, calls 0G Compute, stores results.
// On success, writes a moment.ai_processed DA event so the analysis is on-chain provable.

use std::time::Duration;
use tokio::sync::watch;

use crate::external::compute::ZgComputeClient;
use crate::moments::da_events::{
    MomentDAEventModel, MomentDAEventRepository, EVENT_MOMENT_AI_PROCESSED,
};
use crate::moments::repository::{AiAnalysisUpdate, MomentsRepository};

const POLL_INTERVAL_SECS: u64 = 30;
const BATCH_SIZE: i64 = 5;

pub struct ComputeWorker {
    repo: MomentsRepository,
    compute_client: ZgComputeClient,
    da_event_repo: Option<MomentDAEventRepository>,
    shutdown_rx: watch::Receiver<bool>,
}

impl ComputeWorker {
    pub fn new(
        repo: MomentsRepository,
        compute_client: ZgComputeClient,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            repo,
            compute_client,
            da_event_repo: None,
            shutdown_rx,
        }
    }

    pub fn with_da_events(mut self, repo: MomentDAEventRepository) -> Self {
        self.da_event_repo = Some(repo);
        self
    }

    pub async fn run(mut self) {
        tracing::info!("ComputeWorker started — gameplay intelligence via 0G Compute");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("ComputeWorker received shutdown signal");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)) => {
                    if let Err(e) = self.process_batch().await {
                        tracing::error!(error = %e, "ComputeWorker batch failed");
                    }
                }
            }
        }
    }

    async fn process_batch(&self) -> Result<(), String> {
        let moments = self.repo.find_pending_ai_analysis(BATCH_SIZE).await?;

        for moment in moments {
            // Claim it to prevent concurrent processing
            self.repo.mark_ai_pending(&moment.moment_id).await?;

            match self
                .compute_client
                .analyze_moment(
                    &moment.title,
                    moment.description.as_deref(),
                    &moment.tags,
                    &moment.related_games,
                )
                .await
            {
                Ok(analysis) => {
                    let caption = analysis.caption.chars().take(150).collect::<String>();
                    let rank_score = analysis.rank_score.min(100);
                    let highlights = analysis.highlights.into_iter().take(3).collect::<Vec<_>>();
                    let skill_score = analysis.skill_score.map(|s| s.min(100));

                    let update = AiAnalysisUpdate {
                        caption: caption.clone(),
                        rank_score,
                        highlights: highlights.clone(),
                        moment_type: analysis.moment_type.clone(),
                        skill_score,
                        reaction_quality: analysis.reaction_quality.clone(),
                        rarity: analysis.rarity.clone(),
                    };

                    self.repo
                        .update_ai_analysis(&moment.moment_id, &update)
                        .await?;

                    tracing::info!(
                        moment_id = %moment.moment_id,
                        rank_score = rank_score,
                        moment_type = ?analysis.moment_type,
                        rarity = ?analysis.rarity,
                        "0G Compute gameplay intelligence complete"
                    );

                    // Record this analysis on 0G DA — proves the computation ran
                    if let Some(da_repo) = &self.da_event_repo {
                        let event = MomentDAEventModel::new(
                            &moment.moment_id,
                            EVENT_MOMENT_AI_PROCESSED,
                            &moment.player_wallet_address,
                            serde_json::json!({
                                "rankScore": rank_score,
                                "skillScore": skill_score,
                                "momentType": analysis.moment_type,
                                "rarity": analysis.rarity,
                                "reactionQuality": analysis.reaction_quality,
                                "model": self.compute_client.model(),
                            }),
                        );
                        if let Err(e) = da_repo.create(event).await {
                            tracing::warn!(
                                moment_id = %moment.moment_id,
                                error = %e,
                                "Failed to enqueue ai_processed DA event"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        moment_id = %moment.moment_id,
                        error = %e,
                        "0G Compute gameplay intelligence failed"
                    );
                    self.repo.mark_ai_failed(&moment.moment_id, &e).await?;
                }
            }
        }

        Ok(())
    }
}
