// src/external/bright_data/scrapers/post_scrapers.rs
//
// Bright Data scraper service.
// One generic scrape method internally, per-platform public methods externally.
// scrape_post() is the main public API — returns ScrapedPostData with normalized fields.

use std::time::Duration;

use crate::config::CONFIG;
use crate::handler::AppError;
use crate::moments::social_media::model::platform::Platform;

use super::scraped_post::ScrapedPostData;

/// Bright Data post scraper.
///
/// All methods return `Vec<serde_json::Value>` — the complete raw JSON
/// from BrightData. No typed structs, no field filtering.
///
/// Usage:
/// ```ignore
/// let scraper = BrightDataPostScraper::new();
/// let tweets = scraper.get_twitter_posts(vec!["https://x.com/..."]).await?;
/// let likes = tweets[0]["likes"].as_u64();
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

    pub async fn get_twitter_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_twitter, urls).await
    }

    pub async fn get_instagram_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_instagram, urls)
            .await
    }

    pub async fn get_tiktok_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_tiktok, urls).await
    }

    pub async fn get_facebook_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_facebook, urls)
            .await
    }

    pub async fn get_reddit_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_reddit, urls).await
    }

    pub async fn get_linkedin_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_linkedin, urls)
            .await
    }

    pub async fn get_pinterest_posts(
        &self,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.scrape(&CONFIG.bright_data.dataset_pinterest, urls)
            .await
    }

    // ─── Public API: scrape + normalize ───

    /// Scrape a post and return normalized data.
    ///
    /// This is the method the `social_media` module should call.
    /// It dispatches to the correct BD scraper, then normalizes
    /// platform-specific fields into a `ScrapedPostData`.
    pub async fn scrape_post(
        &self,
        platform: &Platform,
        url: &str,
    ) -> Result<ScrapedPostData, AppError> {
        let urls = vec![url.to_string()];

        let raw_posts = match platform {
            Platform::Twitter => self.get_twitter_posts(urls).await?,
            Platform::Instagram => self.get_instagram_posts(urls).await?,
            Platform::TikTok => self.get_tiktok_posts(urls).await?,
            Platform::Facebook => self.get_facebook_posts(urls).await?,
            Platform::Reddit => self.get_reddit_posts(urls).await?,
            Platform::LinkedIn => self.get_linkedin_posts(urls).await?,
            Platform::Pinterest => self.get_pinterest_posts(urls).await?,
            Platform::Farcaster => {
                return Err(AppError::Internal(
                    "Farcaster scraping not supported".into(),
                ));
            }
        };

        let raw = raw_posts.into_iter().next().ok_or(AppError::Internal(
            "No data returned from BrightData".into(),
        ))?;

        Ok(Self::normalize(platform, raw))
    }

    /// Normalize raw BrightData JSON into a `ScrapedPostData`.
    ///
    /// Maps platform-specific field names to unified fields.
    /// This is where all the "TikTok uses digg_count, Twitter uses likes" knowledge lives.
    fn normalize(platform: &Platform, raw: serde_json::Value) -> ScrapedPostData {
        let error = raw
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let (hashtags_raw, ext_urls, text_field, likes_field) = match platform {
            Platform::Twitter => (
                Self::extract_string_array(&raw, "hashtags"),
                Self::extract_single_url(&raw, "external_url"),
                Self::extract_string(&raw, "description"),
                raw["likes"].as_u64().unwrap_or(0),
            ),
            Platform::Instagram => (
                Self::extract_string_array(&raw, "hashtags"),
                vec![], // No dedicated URL field
                Self::extract_string(&raw, "description"),
                raw["likes"].as_u64().unwrap_or(0),
            ),
            Platform::TikTok => (
                Self::extract_string_array(&raw, "hashtags"),
                vec![], // No dedicated URL field
                Self::extract_string(&raw, "description"),
                raw["digg_count"].as_u64().unwrap_or(0),
            ),
            Platform::Facebook => (
                Self::extract_string_array(&raw, "hashtags"),
                Self::extract_single_url(&raw, "post_external_link"),
                Self::extract_string(&raw, "content"),
                raw["likes"].as_u64().unwrap_or(0),
            ),
            Platform::Reddit => (
                vec![], // Reddit doesn't support hashtags
                Self::extract_string_array(&raw, "embedded_links"),
                Self::extract_string(&raw, "title"),
                raw["num_upvotes"].as_u64().unwrap_or(0),
            ),
            Platform::LinkedIn => (
                Self::extract_string_array(&raw, "hashtags"),
                Self::extract_embedded_links_filtered(&raw),
                Self::extract_string(&raw, "post_text"),
                raw["num_likes"].as_u64().unwrap_or(0),
            ),
            Platform::Pinterest => (
                Self::extract_string_array(&raw, "hashtags"),
                vec![], // BD doesn't expose source links
                Self::extract_string(&raw, "content"),
                raw["likes"].as_u64().unwrap_or(0),
            ),
            Platform::Farcaster => (vec![], vec![], String::new(), 0),
        };

        // Normalize hashtags: strip '#', lowercase
        let hashtags = hashtags_raw
            .into_iter()
            .map(|h| h.trim_start_matches('#').to_lowercase())
            .filter(|h| !h.is_empty())
            .collect();

        ScrapedPostData {
            hashtags,
            external_urls: ext_urls,
            text_content: text_field,
            likes: likes_field as u32,
            error,
            raw,
        }
    }

    // ─── Normalize helpers ───

    /// Extract a string array from a JSON field.
    fn extract_string_array(raw: &serde_json::Value, field: &str) -> Vec<String> {
        raw.get(field)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract a single URL field into a vec (empty if null).
    fn extract_single_url(raw: &serde_json::Value, field: &str) -> Vec<String> {
        raw.get(field)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    }

    /// Extract a string field, defaulting to empty.
    fn extract_string(raw: &serde_json::Value, field: &str) -> String {
        raw.get(field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Extract embedded_links but filter out LinkedIn hashtag links.
    fn extract_embedded_links_filtered(raw: &serde_json::Value) -> Vec<String> {
        Self::extract_string_array(raw, "embedded_links")
            .into_iter()
            .filter(|url| !url.contains("linkedin.com/feed/hashtag/"))
            .filter(|url| !url.contains("linkedin.com/in/"))
            .filter(|url| !url.contains("linkedin.com/company/"))
            .collect()
    }

    // ─── Generic internal: trigger → poll progress → download ───

    async fn scrape(
        &self,
        dataset_id: &str,
        urls: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, AppError> {
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

        // Parse as raw JSON array — no typed struct, get everything BD sends
        serde_json::from_str(&result_text)
            .map_err(|e| AppError::Internal(format!("Parse snapshot: {e}")))
    }
}
