// src/referral/anti_fraud.rs

use crate::redis::ValkyQueue;
use crate::redis::keys::referral_signup_ip_key;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub struct AntiFraudService {
    redis_client: redis::Client,
    verify_queue: ValkyQueue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingReferralCheck {
    pub new_player_id: String,
    pub referred_by_code: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl AntiFraudService {
    pub fn new(redis_client: redis::Client, verify_queue: ValkyQueue) -> Self {
        Self {
            redis_client,
            verify_queue,
        }
    }

    /// Processes a new signup that came with a referral code.
    /// Checks for IP collisions to prevent spam. If safe, pushes to the verification queue.
    pub async fn process_referral_signup(
        &self,
        new_player_id: &str,
        referral_code: &str,
        raw_ip: &str,
    ) -> Result<(), String> {
        // 1. Hash the IP address
        let mut hasher = Sha256::new();
        hasher.update(raw_ip.as_bytes());
        let ip_hash = format!("{:x}", hasher.finalize());

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let collision_key = referral_signup_ip_key(&ip_hash);

        // 2. The 1-Hour Freeze (SETNX algorithm)
        // Set the key ONLY if it does not exist, with a 3600 second (1 hour) TTL
        // Returns 1 if set successfully (clean), or 0 if the key already existed (fraud/spam)
        let is_clean: bool = redis::cmd("SET")
            .arg(&collision_key)
            .arg(1)
            .arg("EX")
            .arg(3600)
            .arg("NX")
            .query_async(&mut conn)
            .await
            .unwrap_or(false); // If true, SETNX succeeded. If false/nil, it failed.

        // 3. Silently drop fraud
        if !is_clean {
            tracing::warn!(
                "Anti-Fraud blocked a duplicate signup reward from the same IP hash: {}",
                ip_hash
            );
            return Ok(()); // We don't return an error to the user, we just don't grant the referral
        }

        // 4. IP is clean. Push to async Verification Queue
        let event = PendingReferralCheck {
            new_player_id: new_player_id.to_string(),
            referred_by_code: referral_code.to_string(),
            timestamp: Utc::now(),
        };

        self.verify_queue
            .push_async(&event)
            .await
            .map_err(|e| format!("Failed to push to verify queue: {}", e))?;

        Ok(())
    }
}
