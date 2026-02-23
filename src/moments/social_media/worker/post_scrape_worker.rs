// src/moments/social_media/worker/post_scrape_worker.rs
// Singleton PostScrapeWorker - consumes ScrapeJobs from Valkey queue
// Enforces 24h delay window, then scrapes and validates posts

use std::time::Duration;

use chrono::Utc;

use crate::config::CONFIG;
use crate::moments::social_media::repository::post_repository::PostRepository;
use crate::moments::social_media::service::post_scraper_service::PostScraperService;
use crate::redis::ValkyQueue;

use super::scrape_job::{ScrapeJob, SCRAPE_DEAD_LETTER, SCRAPE_QUEUE};

/// Singleton worker that processes post scrape jobs.
///
/// Mirrors the MigrationWorker pattern:
/// - Poll queue via BRPOP
/// - Configurable age gate: if the post is younger than min_age_hours, re-push and move on
/// - Process: scrape the post via BrightData, validate, update metrics
/// - Retry / dead-letter on failure
pub struct PostScrapeWorker {
    queue: ValkyQueue,
    scraper_service: PostScraperService,
}

impl PostScrapeWorker {
    /// Create the worker (call only once).
    pub fn new(queue: ValkyQueue, repo: PostRepository) -> Self {
        Self {
            queue,
            scraper_service: PostScraperService::new(repo),
        }
    }

    /// Run the worker loop. Blocks forever, processing jobs as they come.
    pub async fn run(&self) {
        tracing::info!(
            queue = SCRAPE_QUEUE,
            min_age_hours = CONFIG.scrape.min_age_hours,
            max_retries = CONFIG.scrape.max_retries,
            "PostScrapeWorker started"
        );

        loop {
            match self.queue.pop::<ScrapeJob>(CONFIG.scrape.poll_timeout_secs) {
                Ok(Some(job)) => {
                    tracing::info!(
                        post_db_id = %job.post_db_id,
                        platform = ?job.platform,
                        attempt = job.attempt,
                        "Received scrape job"
                    );

                    // Configurable age gate: check if enough time has passed
                    let age = Utc::now() - job.created_at;
                    if age.num_hours() < CONFIG.scrape.min_age_hours {
                        let remaining = CONFIG.scrape.min_age_hours - age.num_hours();
                        tracing::info!(
                            post_db_id = %job.post_db_id,
                            remaining_hours = remaining,
                            "Post too young — re-queuing for later"
                        );
                        // Re-push to back of queue so other ready jobs can be processed first
                        if let Err(e) = self.queue.push(&job) {
                            tracing::error!(error = %e, "Failed to re-queue young job");
                        }
                        // Sleep briefly to avoid tight looping on the same job
                        tokio::time::sleep(Duration::from_secs(CONFIG.scrape.requeue_sleep_secs))
                            .await;
                        continue;
                    }

                    // Post is old enough — process it
                    self.process_job(job).await;
                }
                Ok(None) => {
                    // Timeout — no jobs available, loop back and wait again
                }
                Err(e) => {
                    tracing::error!(error = %e, "Queue pop error, retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Process a single scrape job:
    /// 1. Call BrightData scraper via PostScraperService
    /// 2. Validate the post and extract engagement metrics
    /// 3. Update metrics + validation status in MongoDB
    async fn process_job(&self, job: ScrapeJob) {
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
            if let Err(e) = self.queue.push(&retry_job) {
                tracing::error!(error = %e, "Failed to re-queue job");
            }
        } else {
            tracing::error!(
                post_db_id = %job.post_db_id,
                attempts = job.attempt,
                "Job failed after max retries — sending to dead letter queue"
            );
            let dlq = ValkyQueue::new(self.queue.connection().clone(), SCRAPE_DEAD_LETTER);
            if let Err(e) = dlq.push(&job) {
                tracing::error!(error = %e, "Failed to push to dead letter queue");
            }
        }
    }
}
