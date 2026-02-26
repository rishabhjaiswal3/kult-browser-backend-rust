// src/moments/social_media/service/post_scraper_service.rs
//
// The "brain" of the scrape pipeline.
// Called by PostScrapeWorker after the 24h gate passes.
// Dispatches to BrightDataPostScraper.scrape_post() which returns
// normalized ScrapedPostData, then validates via PostValidator.

use crate::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;
use crate::moments::social_media::model::platform::Platform;
use crate::moments::social_media::model::post_model::ValidationStatus;
use crate::moments::social_media::repository::post_repository::PostRepository;
use crate::moments::social_media::service::post_validator::PostValidator;
use mongodb::bson::oid::ObjectId;

/// Result of scraping and validating a single post
#[derive(Debug)]
pub struct ScrapeResult {
    pub likes: u32,
    pub score: u32,
    pub is_valid: bool,
    pub status: ValidationStatus,
    pub validation_reason: String,
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
    /// Flow:
    /// 1. `scraper.scrape_post()` — dispatches to BD, returns `ScrapedPostData` (normalized)
    /// 2. `PostValidator::validate()` — checks hashtags/URLs/text for Kult signals
    /// 3. Score from likes
    /// 4. Update MongoDB
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

        // Step 1: Scrape + normalize (BD module handles platform dispatch + field mapping)
        let scraped = self
            .scraper
            .scrape_post(platform, url)
            .await
            .map_err(|e| format!("Scrape failed: {}", e))?;

        // Step 2: Validate (domain layer — checks for Kult hashtags/URLs/text)
        let (is_kult_post, reason) = PostValidator::validate(&scraped);

        let result = if is_kult_post {
            tracing::info!(
                post_db_id = %post_db_id,
                likes = scraped.likes,
                reason = %reason,
                "Post validated as Kult post"
            );
            ScrapeResult {
                likes: scraped.likes,
                score: Self::calculate_score(scraped.likes),
                is_valid: true,
                status: ValidationStatus::Valid,
                validation_reason: reason.to_string(),
            }
        } else {
            tracing::warn!(
                post_db_id = %post_db_id,
                reason = %reason,
                "Post is NOT a Kult post"
            );
            ScrapeResult {
                likes: scraped.likes,
                score: 0,
                is_valid: true, // We DID validate it — it just failed
                status: ValidationStatus::Invalid,
                validation_reason: reason.to_string(),
            }
        };

        // Step 3: Update MongoDB
        self.repo
            .update_post_metrics(
                post_db_id,
                result.likes,
                result.score,
                result.is_valid,
                result.status.clone(),
                &result.validation_reason,
            )
            .await
            .map_err(|e| format!("Failed to update post metrics: {}", e))?;

        tracing::info!(
            post_db_id = %post_db_id,
            likes = result.likes,
            score = result.score,
            status = ?result.status,
            reason = %result.validation_reason,
            "Post validation complete — MongoDB updated"
        );

        Ok(result)
    }

    /// Calculate a score from the raw likes count.
    ///
    /// For now this is a simple 1:1 mapping.
    fn calculate_score(likes: u32) -> u32 {
        likes
    }
}
