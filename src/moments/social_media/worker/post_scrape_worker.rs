// src/moments/social_media/worker/post_scrape_worker.rs
// Singleton PostScrapeWorker - consumes ScrapeJobs from Valkey queue
// Enforces 24h delay window, then scrapes and validates posts

use chrono::Utc;
use std::time::Duration;
use tokio::sync::watch;

use super::scrape_job::{ScrapeJob, SCRAPE_DEAD_LETTER, SCRAPE_QUEUE};
use crate::config::CONFIG;
use crate::moments::social_media::repository::post_repository::PostRepository;
use crate::moments::social_media::service::post_scraper_service::PostScraperService;
use crate::redis::ValkyQueue;

/// Singleton worker that processes post scrape jobs.
///
/// Converts to Reliable Queue pattern:
/// - Polls queue via `pop_async` (BRPOPLPUSH)
/// - Configurable age gate: if the post is younger than min_age_hours, re-push and move on
/// - Scrapes the post via BrightData, validates, updates metrics
/// - Unconditionally `ack_async`s the raw payload when done (even on retry/DLQ)
/// - Listens for shutdown signal to exit gracefully
pub struct PostScrapeWorker {
    queue: ValkyQueue,
    scraper_service: PostScraperService,
    shutdown_rx: watch::Receiver<bool>,
}

impl PostScrapeWorker {
    /// Create the worker.
    pub fn new(
        queue: ValkyQueue,
        repo: PostRepository,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            queue,
            scraper_service: PostScraperService::new(repo),
            shutdown_rx,
        }
    }

    /// Run the worker loop until a shutdown signal is received.
    pub async fn run(mut self) {
        tracing::info!(
            queue = SCRAPE_QUEUE,
            min_age_hours = CONFIG.scrape.min_age_hours,
            max_retries = CONFIG.scrape.max_retries,
            "PostScrapeWorker started"
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
                        tracing::info!("PostScrapeWorker received shutdown signal. Exiting gracefully.");
                        break;
                    }
                }

                // 2. Poll the queue
                pop_result = self.queue.pop_async::<ScrapeJob>(CONFIG.scrape.poll_timeout_secs) => {
                    match pop_result {
                        Ok(Some((job, raw_data))) => {
                            tracing::info!(
                                post_db_id = %job.post_db_id,
                                platform = ?job.platform,
                                attempt = job.attempt,
                                "Received scrape job"
                            );

                            // Process the job logic...
                            self.handle_popped_job(job, &raw_data).await;
                        }
                        Ok(None) => continue, // Timeout, loop back
                        Err(e) => {
                            tracing::error!(error = %e, "Queue pop error, retrying in 5s");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    }

    /// Extracted logic for handling a single popped job, including age gate and acknowledging.
    async fn handle_popped_job(&self, job: ScrapeJob, raw_data: &str) {
        let age = Utc::now() - job.created_at;

        if age.num_hours() < CONFIG.scrape.min_age_hours {
            let remaining = CONFIG.scrape.min_age_hours - age.num_hours();
            tracing::info!(
                post_db_id = %job.post_db_id,
                remaining_hours = remaining,
                "Post too young — re-queuing for later"
            );

            // Push back to main queue
            if let Err(e) = self.queue.push_async(&job).await {
                tracing::error!(error = %e, "Failed to re-queue young job");
            }
            // Acknowledge the old raw payload so it drops from processing queue
            if let Err(e) = self.queue.ack_async(raw_data).await {
                tracing::error!(error = %e, "Failed to ACK young job");
            }

            tokio::time::sleep(Duration::from_secs(CONFIG.scrape.requeue_sleep_secs)).await;
            return;
        }

        // Post is old enough — process it
        match self
            .scraper_service
            .scrape_and_validate(job.post_db_id, &job.platform, &job.url)
            .await
        {
            Ok(result) => {
                tracing::info!(
                    post_db_id = %job.post_db_id,
                    likes = result.likes,
                    score = result.score,
                    status = ?result.status,
                    "Scrape job completed successfully"
                );
            }
            Err(e) => {
                tracing::error!(
                    post_db_id = %job.post_db_id,
                    error = %e,
                    "Scrape job failed"
                );
                self.handle_failure(job).await;
            }
        }

        // UNCONDITIONAL ACK: Whether successful, retried, or DLQ'd, the original popped job is done.
        if let Err(e) = self.queue.ack_async(raw_data).await {
            tracing::error!(error = %e, "Failed to ACK completed job");
        }
    }

    /// Handle a failed job: retry or send to dead-letter queue.
    async fn handle_failure(&self, job: ScrapeJob) {
        if job.attempt < CONFIG.scrape.max_retries {
            let retry_job = ScrapeJob {
                attempt: job.attempt + 1,
                ..job
            };
            tracing::warn!(
                post_db_id = %retry_job.post_db_id,
                attempt = retry_job.attempt,
                "Retrying scrape job"
            );
            if let Err(e) = self.queue.push_async(&retry_job).await {
                tracing::error!(error = %e, "Failed to re-queue job");
            }
        } else {
            tracing::error!(
                post_db_id = %job.post_db_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            let dlq = ValkyQueue::new(self.queue.connection().clone(), SCRAPE_DEAD_LETTER);
            if let Err(e) = dlq.push_async(&job).await {
                tracing::error!(error = %e, "Failed to push to dead letter queue");
            }
        }
    }
}
