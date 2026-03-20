// src/referral/analytics.rs

use crate::redis::ValkyQueue;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub struct ClickAnalyticsService {
    click_queue: Option<ValkyQueue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClickEvent {
    pub code: String,
    pub ip_hash: String,
    pub user_agent: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl ClickAnalyticsService {
    pub fn new(click_queue: Option<ValkyQueue>) -> Self {
        Self { click_queue }
    }

    /// Hashes the raw IP address with today's date to anonymize it,
    /// then asynchronously pushes the click event into Valkey for the background worker to process.
    pub async fn log_click(&self, code: &str, raw_ip: &str, user_agent: &str) {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        // Hash the IP with a daily salt to prevent PII storage while allowing daily unique counting
        let mut hasher = Sha256::new();
        hasher.update(raw_ip.as_bytes());
        hasher.update(today.as_bytes());
        let ip_hash = format!("{:x}", hasher.finalize());

        let event = ClickEvent {
            code: code.to_string(),
            ip_hash,
            user_agent: user_agent.to_string(),
            timestamp: Utc::now(),
        };

        let Some(queue) = self.click_queue.clone() else {
            tracing::debug!(code = %code, "Referral click queue unavailable; skipping analytics enqueue");
            return;
        };

        // Fire and forget - don't block the HTTP response if Redis is slow
        tokio::spawn(async move {
            if let Err(e) = queue.push_async(&event).await {
                tracing::error!("Failed to enqueue referral click: {}", e);
            }
        });
    }
}
