// src/moments/social_media/service/post_scraper_service.rs
//
// The "brain" of the scrape pipeline.
// Called by PostScrapeWorker after the 24h gate passes.
// Dispatches to the correct BrightData scraper based on platform,
// extracts engagement metrics, validates the post, and updates MongoDB.

use crate::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;
use crate::moments::social_media::model::platform::Platform;
use crate::moments::social_media::model::post_model::ValidationStatus;
use crate::moments::social_media::repository::post_repository::PostRepository;
use mongodb::bson::oid::ObjectId;

/// Result of scraping a single post
#[derive(Debug)]
pub struct ScrapeResult {
    pub likes: u32,
    pub score: u32,
    pub is_valid: bool,
    pub status: ValidationStatus,
}

/// Service that orchestrates scraping and validation for shared posts.
#[derive(Clone)]
pub struct PostScraperService {
    scraper: BrightDataPostScraper,
    repo: PostRepository,
}

impl PostScraperService {
    pub fn new(repo: PostRepository) -> Self {
        Self {
            scraper: BrightDataPostScraper::new(),
            repo,
        }
    }

    /// Scrape a post by platform, validate it, calculate score, and update MongoDB.
    ///
    /// Returns Ok(ScrapeResult) on success, Err on infrastructure failure.
    pub async fn scrape_and_validate(
        &self,
        post_db_id: ObjectId,
        platform: &Platform,
        url: &str,
    ) -> Result<ScrapeResult, String> {
        tracing::info!(
            post_db_id = %post_db_id,
            platform = ?platform,
            url = %url,
            "Scraping post via BrightData"
        );

        // Step 1: Call the correct BrightData scraper based on platform
        let scrape_result = self.scrape_by_platform(platform, url).await;

        let result = match scrape_result {
            Ok(likes) => {
                tracing::info!(
                    post_db_id = %post_db_id,
                    likes = likes,
                    "Scrape successful — post exists"
                );
                ScrapeResult {
                    likes,
                    score: Self::calculate_score(likes),
                    is_valid: true,
                    status: ValidationStatus::Valid,
                }
            }
            Err(e) => {
                tracing::warn!(
                    post_db_id = %post_db_id,
                    error = %e,
                    "Scrape failed or post not found — marking Invalid"
                );
                ScrapeResult {
                    likes: 0,
                    score: 0,
                    is_valid: true, // We DID validate it — it just failed validation
                    status: ValidationStatus::Invalid,
                }
            }
        };

        // Step 2: Update MongoDB with the scraped data
        self.repo
            .update_post_metrics(
                post_db_id,
                result.likes,
                result.score,
                result.is_valid,
                result.status.clone(),
            )
            .await
            .map_err(|e| format!("Failed to update post metrics: {}", e))?;

        tracing::info!(
            post_db_id = %post_db_id,
            likes = result.likes,
            score = result.score,
            status = ?result.status,
            "Post validation complete — MongoDB updated"
        );

        Ok(result)
    }

    /// Dispatch to the correct BrightData scraper and extract the likes count.
    /// Returns Ok(likes) if the post was found, Err if deleted/not found.
    async fn scrape_by_platform(&self, platform: &Platform, url: &str) -> Result<u32, String> {
        let urls = vec![url.to_string()];

        match platform {
            Platform::Twitter => {
                let posts = self
                    .scraper
                    .get_twitter_posts(urls)
                    .await
                    .map_err(|e| format!("Twitter scrape failed: {}", e))?;
                let post = posts.first().ok_or("No Twitter data returned")?;
                if post.error.is_some() {
                    return Err(format!("Twitter post error: {:?}", post.error));
                }
                Ok(post.likes.unwrap_or(0) as u32)
            }
            Platform::Pinterest => {
                let posts = self
                    .scraper
                    .get_pinterest_posts(urls)
                    .await
                    .map_err(|e| format!("Pinterest scrape failed: {}", e))?;
                let post = posts.first().ok_or("No Pinterest data returned")?;
                if post.error.is_some() {
                    return Err(format!("Pinterest post error: {:?}", post.error));
                }
                Ok(post.likes.unwrap_or(0) as u32)
            }
            Platform::Farcaster => {
                // Farcaster is not yet supported by BrightData
                tracing::warn!("Farcaster scraping not yet supported — defaulting to 0 likes");
                Ok(0)
            }
        }
    }

    /// Calculate a score from the raw likes count.
    ///
    /// For now this is a simple 1:1 mapping.
    /// In the future, this can factor in platform weight multipliers,
    /// time decay, engagement rate, etc.
    fn calculate_score(likes: u32) -> u32 {
        likes
    }
}
