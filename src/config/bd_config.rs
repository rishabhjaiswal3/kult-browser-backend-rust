// src/config/bd_config.rs
// Bright Data Web Scraper API configuration

use std::env;

/// Bright Data API configuration
#[derive(Debug, Clone)]
pub struct BdConfig {
    /// API key for authentication (Bearer token)
    pub api_key: String,
    /// Base URL for the Bright Data API
    pub base_url: String,
    /// API path for triggering scrape jobs
    pub trigger_path: String,
    /// API path for checking job progress/status
    pub progress_path: String,
    /// API path for downloading snapshot results
    pub snapshot_path: String,
    /// Polling interval in seconds when waiting for snapshot results
    pub poll_interval_secs: u64,
    /// Maximum time in seconds to wait for a snapshot before timing out
    pub poll_timeout_secs: u64,

    // Dataset IDs per platform (each routes to a different scraper engine)
    pub dataset_twitter: String,
    pub dataset_instagram: String,
    pub dataset_tiktok: String,
    pub dataset_facebook: String,
    pub dataset_reddit: String,
    pub dataset_linkedin: String,
    pub dataset_pinterest: String,
}

impl BdConfig {
    /// Load Bright Data config from environment variables.
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("BD_API_KEY").expect("BD_API_KEY must be set"),
            base_url: env::var("BD_BASE_URL")
                .unwrap_or_else(|_| "https://api.brightdata.com".to_string()),
            trigger_path: env::var("BD_TRIGGER_PATH")
                .unwrap_or_else(|_| "/datasets/v3/trigger".to_string()),
            progress_path: env::var("BD_PROGRESS_PATH")
                .unwrap_or_else(|_| "/datasets/v3/progress".to_string()),
            snapshot_path: env::var("BD_SNAPSHOT_PATH")
                .unwrap_or_else(|_| "/datasets/v3/snapshot".to_string()),
            poll_interval_secs: env::var("BD_POLL_INTERVAL")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("BD_POLL_INTERVAL must be a number"),
            poll_timeout_secs: env::var("BD_POLL_TIMEOUT")
                .unwrap_or_else(|_| "180".to_string())
                .parse()
                .expect("BD_POLL_TIMEOUT must be a number"),
            dataset_twitter: env::var("BD_DATASET_TWITTER")
                .unwrap_or_else(|_| "gd_lwxkxvnf1cynvib9co".to_string()),
            dataset_instagram: env::var("BD_DATASET_INSTAGRAM")
                .unwrap_or_else(|_| "gd_lk5ns7kz21pck8jpis".to_string()),
            dataset_tiktok: env::var("BD_DATASET_TIKTOK")
                .unwrap_or_else(|_| "gd_lu702nij2f790tmv9h".to_string()),
            dataset_facebook: env::var("BD_DATASET_FACEBOOK")
                .unwrap_or_else(|_| "gd_lyclm1571iy3mv57zw".to_string()),
            dataset_reddit: env::var("BD_DATASET_REDDIT")
                .unwrap_or_else(|_| "gd_lvz8ah06191smkebj4".to_string()),
            dataset_linkedin: env::var("BD_DATASET_LINKEDIN")
                .unwrap_or_else(|_| "gd_lyy3tktm25m4avu764".to_string()),
            dataset_pinterest: env::var("BD_DATASET_PINTEREST")
                .unwrap_or_else(|_| "gd_lk0sjs4d21kdr7cnlv".to_string()),
        }
    }

    /// Build the full trigger URL for a given dataset.
    pub fn trigger_url(&self, dataset_id: &str) -> String {
        format!(
            "{}{}?dataset_id={}&include_errors=true",
            self.base_url, self.trigger_path, dataset_id
        )
    }

    /// Build the full progress poll URL.
    pub fn progress_url(&self, snapshot_id: &str) -> String {
        format!("{}{}/{}", self.base_url, self.progress_path, snapshot_id)
    }

    /// Build the full snapshot download URL.
    pub fn snapshot_url(&self, snapshot_id: &str) -> String {
        format!(
            "{}{}/{}?format=json",
            self.base_url, self.snapshot_path, snapshot_id
        )
    }
}
