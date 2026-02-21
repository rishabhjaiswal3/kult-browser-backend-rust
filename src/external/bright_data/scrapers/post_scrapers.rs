// src/external/bright_data/scrapers/post_scrapers.rs
//
// Bright Data scraper service.
// One generic scrape method internally, per-platform public methods externally.
// All BD details (dataset IDs, API paths, polling) are hidden from callers.

use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::config::CONFIG;
use crate::handler::AppError;

use super::models::*;

/// Bright Data post scraper.
///
/// Usage from moments/social_media:
/// ```ignore
/// let scraper = BrightDataPostScraper::new();
/// let tweets = scraper.get_twitter_posts(vec!["https://x.com/..."]).await?;
/// let grams  = scraper.get_instagram_posts(vec!["https://instagram.com/..."]).await?;
/// ```
#[derive(Clone)]
pub struct BrightDataPostScraper {
    client: reqwest::Client,
}

impl BrightDataPostScraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    CONFIG.bright_data.poll_timeout_secs + 30,
                ))
                .build()
                .expect("Failed to create reqwest client"),
        }
    }

    // ─── Per-platform public methods ─────────

    pub async fn get_twitter_posts(&self, urls: Vec<String>) -> Result<Vec<TwitterPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_twitter, urls).await
    }

    pub async fn get_instagram_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<InstagramPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_instagram, urls)
            .await
    }

    pub async fn get_tiktok_posts(&self, urls: Vec<String>) -> Result<Vec<TikTokPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_tiktok, urls).await
    }

    pub async fn get_facebook_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<FacebookPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_facebook, urls)
            .await
    }

    pub async fn get_reddit_posts(&self, urls: Vec<String>) -> Result<Vec<RedditPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_reddit, urls).await
    }

    pub async fn get_linkedin_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<LinkedInPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_linkedin, urls)
            .await
    }

    pub async fn get_pinterest_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<PinterestPost>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_pinterest, urls)
            .await
    }

    // ─── Generic internal: trigger → poll progress → download ───

    async fn scrape<T: DeserializeOwned>(
        &self,
        dataset_id: &str,
        urls: Vec<String>,
    ) -> Result<Vec<T>, AppError> {
        let cfg = &CONFIG.bright_data;

        tracing::info!(%dataset_id, count = urls.len(), "Triggering scrape");

        // Step 1: Trigger
        let body: Vec<serde_json::Value> =
            urls.iter().map(|u| serde_json::json!({"url": u})).collect();

        let resp = self
            .client
            .post(cfg.trigger_url(dataset_id))
            .header("Authorization", format!("Bearer {}", cfg.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Trigger failed: {e}")))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::Internal(format!("Trigger {status}: {text}")));
        }

        #[derive(serde::Deserialize)]
        struct TriggerResp {
            snapshot_id: String,
        }
        let snap: TriggerResp = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(format!("Parse trigger: {e}")))?;

        tracing::info!(snapshot_id = %snap.snapshot_id, "Scrape triggered, polling progress");

        // Step 2: Poll progress (GET /datasets/v3/progress/{id})
        let interval = Duration::from_secs(cfg.poll_interval_secs);
        let timeout = Duration::from_secs(cfg.poll_timeout_secs);
        let start = std::time::Instant::now();

        loop {
            let resp_text = self
                .client
                .get(cfg.progress_url(&snap.snapshot_id))
                .header("Authorization", format!("Bearer {}", cfg.api_key))
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Progress poll failed: {e}")))?
                .text()
                .await
                .unwrap_or_default();

            #[derive(serde::Deserialize)]
            struct ProgressResp {
                status: String,
            }
            let progress: ProgressResp = serde_json::from_str(&resp_text)
                .map_err(|e| AppError::Internal(format!("Parse progress: {e}")))?;

            match progress.status.as_str() {
                "ready" => break,
                "failed" => {
                    return Err(AppError::Internal(format!(
                        "Snapshot {} failed: {resp_text}",
                        snap.snapshot_id
                    )));
                }
                // "starting" | "running"
                _ => {
                    if start.elapsed() > timeout {
                        return Err(AppError::Internal(format!(
                            "Snapshot {} timed out after {}s",
                            snap.snapshot_id,
                            timeout.as_secs()
                        )));
                    }
                    tracing::debug!(
                        snapshot_id = %snap.snapshot_id,
                        status = %progress.status,
                        "Waiting..."
                    );
                    tokio::time::sleep(interval).await;
                }
            }
        }

        // Step 3: Download results (GET /datasets/v3/snapshot/{id}?format=json)
        let result_text = self
            .client
            .get(cfg.snapshot_url(&snap.snapshot_id))
            .header("Authorization", format!("Bearer {}", cfg.api_key))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Snapshot download failed: {e}")))?
            .text()
            .await
            .unwrap_or_default();

        serde_json::from_str(&result_text)
            .map_err(|e| AppError::Internal(format!("Parse snapshot: {e}")))
    }
}
