// src/config/scrape_config.rs
// Configuration for the post scrape worker pipeline

use std::env;

/// Post scrape worker configuration
#[derive(Debug, Clone)]
pub struct ScrapeConfig {
    /// Minimum age (in hours) a post must be before scraping (default: 24)
    pub min_age_hours: i64,
    /// Maximum retry attempts before sending to dead-letter queue (default: 3)
    pub max_retries: u32,
    /// Queue poll timeout in seconds (default: 5)
    pub poll_timeout_secs: u32,
    /// Sleep duration (in seconds) after re-queuing a young job (default: 60)
    pub requeue_sleep_secs: u64,
}

impl ScrapeConfig {
    /// Load scrape config from environment variables.
    pub fn from_env() -> Self {
        Self {
            min_age_hours: env::var("SCRAPE_MIN_AGE_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("SCRAPE_MIN_AGE_HOURS must be a valid i64"),

            max_retries: env::var("SCRAPE_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .expect("SCRAPE_MAX_RETRIES must be a valid u32"),

            poll_timeout_secs: env::var("SCRAPE_POLL_TIMEOUT_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("SCRAPE_POLL_TIMEOUT_SECS must be a valid u32"),

            requeue_sleep_secs: env::var("SCRAPE_REQUEUE_SLEEP_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("SCRAPE_REQUEUE_SLEEP_SECS must be a valid u64"),
        }
    }
}
