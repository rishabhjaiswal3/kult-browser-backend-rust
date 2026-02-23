// src/moments/social_media/worker/scrape_job.rs
// Job payload for post scrape queue

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::moments::social_media::model::platform::Platform;

/// Queue name used for post scraping jobs.
pub const SCRAPE_QUEUE: &str = "posts:scrape";

/// Dead-letter queue for permanently failed scrape jobs.
pub const SCRAPE_DEAD_LETTER: &str = "posts:scrape:dead";

/// Job payload pushed by PostService, popped by PostScrapeWorker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeJob {
    /// MongoDB _id of the SharedPost document
    pub post_db_id: ObjectId,
    /// Social media platform
    pub platform: Platform,
    /// The social media post URL to scrape
    pub url: String,
    /// When the SharedPost was originally created (used for 24h gate)
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    /// Retry attempt number
    pub attempt: u32,
}
