// src/config/scrape_config.rs
// Configuration for the post scrape worker pipeline

use std::env;

/// Post scrape worker configuration
#[derive(Debug, Clone)]
pub struct ScrapeConfig {
    /// Minimum age (in seconds) a post must be before scraping (default: 24h)
    pub min_age_secs: i64,
    /// Maximum retry attempts before sending to dead-letter queue (default: 3)
    pub max_retries: u32,
    /// Queue poll timeout in seconds (default: 5)
    pub poll_timeout_secs: u32,
    /// Sleep duration (in seconds) after re-queuing a young job (default: 60)
    pub requeue_sleep_secs: u64,
    /// Validation keywords shared across hashtag / URL / text checks.
    ///
    /// Declared via `SCRAPE_VALIDATION_TERMS` in `.env`.
    pub validation_terms: Vec<String>,
}

impl ScrapeConfig {
    /// Load scrape config from environment variables.
    pub fn from_env() -> Self {
        let min_age_secs = env::var("SCRAPE_MIN_AGE_SECS")
            .ok()
            .or_else(|| {
                env::var("SCRAPE_MIN_AGE_HOURS")
                    .ok()
                    .map(|hours| match hours.parse::<i64>() {
                        Ok(parsed) => (parsed * 3600).to_string(),
                        Err(_) => hours,
                    })
            })
            .unwrap_or_else(|| (24 * 60 * 60).to_string())
            .parse()
            .expect("SCRAPE_MIN_AGE_SECS must be a valid i64");

        let validation_terms = env::var("SCRAPE_VALIDATION_TERMS")
            .unwrap_or_else(|_| "game".to_string())
            .split(',')
            .map(|term| term.trim().to_lowercase())
            .filter(|term| !term.is_empty())
            .collect::<Vec<_>>();

        Self {
            min_age_secs,

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

            validation_terms,
        }
    }
}
