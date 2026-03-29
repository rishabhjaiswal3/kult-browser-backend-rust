use crate::middleware::auth::model::JwtClaims;
use crate::player::model::PlayerModel;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

/// JWT signing and verification service.
pub struct AuthService;

impl AuthService {
    /// Get the JWT secret from environment.
    /// Falls back to a dev-only default (NEVER use in production).
    fn get_secret() -> String {
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
    }

    /// Get token expiration duration (in days) from environment.
    fn get_expiration_days() -> i64 {
        env::var("JWT_EXPIRATION_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7) // Default: 7 days
    }

    /// Sign a JWT token for the given player.
    ///
    /// # Arguments
    /// * `player` - The PlayerModel to create a token for
    ///
    /// # Returns
    /// * JWT token string
    pub fn sign_token(player: &PlayerModel) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + Duration::days(Self::get_expiration_days());

        let claims = JwtClaims {
            walletAddress: player.wallet_address.clone(),
            player_id: player.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            name: player.name.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        let secret = Self::get_secret();
        let key = EncodingKey::from_secret(secret.as_bytes());

        encode(&Header::default(), &claims, &key)
            .map_err(|e| format!("Failed to sign token: {}", e))
    }

    /// Verify and decode a JWT token.
    ///
    /// # Arguments
    /// * `token` - The JWT token string (without "Bearer " prefix)
    ///
    /// # Returns
    /// * Decoded JwtClaims if valid
    pub fn verify_token(token: &str) -> Result<JwtClaims, String> {
        let secret = Self::get_secret();
        let key = DecodingKey::from_secret(secret.as_bytes());

        let validation = Validation::default();

        decode::<JwtClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| format!("Invalid token: {}", e))
    }
}
