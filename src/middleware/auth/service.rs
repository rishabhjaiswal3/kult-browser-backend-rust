use crate::config::CONFIG;
use crate::middleware::auth::model::JwtClaims;
use crate::player::model::PlayerModel;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// JWT signing and verification service.
/// Uses cached config values from CONFIG (loaded once at startup).
pub struct AuthService;

impl AuthService {
    /// Sign a JWT token for the given player.
    pub fn sign_token(player: &PlayerModel) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + Duration::days(CONFIG.auth.jwt_expiration_days);

        let claims = JwtClaims {
            wallet_address: player.wallet_address.clone(),
            player_id: player.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            name: player.name.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        let key = EncodingKey::from_secret(CONFIG.auth.jwt_secret.as_bytes());

        encode(&Header::default(), &claims, &key)
            .map_err(|e| format!("Failed to sign token: {}", e))
    }

    /// Verify and decode a JWT token.
    pub fn verify_token(token: &str) -> Result<JwtClaims, String> {
        let key = DecodingKey::from_secret(CONFIG.auth.jwt_secret.as_bytes());
        let validation = Validation::default();

        decode::<JwtClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| format!("Invalid token: {}", e))
    }
}
