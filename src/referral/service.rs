// src/referral/service.rs

use crate::player::repository::PlayerRepository;
use crate::redis::keys::referral_code_cache_key;
use nanoid::nanoid;
use redis::AsyncCommands;
use std::sync::Arc;

pub struct ReferralService {
    player_repo: Arc<PlayerRepository>,
    redis_client: Option<redis::Client>,
}

impl ReferralService {
    pub fn new(player_repo: Arc<PlayerRepository>, redis_client: Option<redis::Client>) -> Self {
        Self {
            player_repo,
            redis_client,
        }
    }

    /// Fetches the authenticated player's referral code.
    /// If they don't have one, generates it, saves it to MongoDB, and caches it in Valkey.
    pub async fn get_or_create_code(&self, wallet_address: &str) -> Result<String, String> {
        let normalized_wallet = wallet_address.trim().to_lowercase();

        // 1. Fetch existing player
        let player = match self.player_repo.find_by_wallet(&normalized_wallet).await? {
            Some(p) => p,
            None => return Err("Player not found".to_string()),
        };

        // 2. Return existing code if present
        if let Some(code) = player.referral_code {
            return Ok(code);
        }

        // 3. Generate a new 8-character url-safe nanoid
        // We use a custom alphabet to avoid ambiguous characters
        let alphabet: [char; 36] = [
            '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
            'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
            'C', 'D',
        ];
        let new_code = nanoid!(8, &alphabet);

        // 4. Update MongoDB Player Document
        self.player_repo
            .set_referral_code(&normalized_wallet, &new_code)
            .await?;

        let cache_key = referral_code_cache_key(&new_code);
        self.try_cache_referral_code(&cache_key, &normalized_wallet)
            .await;

        Ok(new_code)
    }

    /// Lookup a wallet address from a given referral code, using Redis cache first, falling back to Mongo.
    pub async fn resolve_code_to_wallet(&self, code: &str) -> Result<Option<String>, String> {
        let cache_key = referral_code_cache_key(code);

        // 1. Check Redis Cache when available
        if let Some(redis_client) = &self.redis_client {
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let cached_wallet: Option<String> =
                        conn.get(&cache_key).await.map_err(|e| {
                            tracing::warn!(error = %e, code = %code, "Referral cache get failed");
                            format!("Redis get error: {}", e)
                        })?;

                    if let Some(wallet) = cached_wallet {
                        return Ok(Some(wallet));
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, code = %code, "Referral cache unavailable, falling back to MongoDB");
                }
            }
        }

        // 2. Cache miss: Fallback to MongoDB
        let player = self.player_repo.find_by_referral_code(code).await?;

        match player {
            Some(p) => {
                self.try_cache_referral_code(&cache_key, &p.wallet_address)
                    .await;

                Ok(Some(p.wallet_address))
            }
            None => Ok(None),
        }
    }

    async fn try_cache_referral_code(&self, cache_key: &str, wallet_address: &str) {
        let Some(redis_client) = &self.redis_client else {
            return;
        };

        match redis_client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                if let Err(e) = conn.set::<_, _, ()>(cache_key, wallet_address).await {
                    tracing::warn!(
                        error = %e,
                        cache_key = %cache_key,
                        "Failed to cache referral code in Redis"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    cache_key = %cache_key,
                    "Referral cache connection unavailable"
                );
            }
        }
    }
}
