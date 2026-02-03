// src/auth/model/jwt_claims.rs

use serde::{Deserialize, Serialize};

/// JWT payload claims for player authentication.
/// 
/// Industry Standard Fields (RFC 7519):
/// - `sub`: Subject - the player's wallet address
/// - `iat`: Issued At - Unix timestamp
/// - `exp`: Expiration - Unix timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject: wallet address (primary identifier)
    pub sub: String,

    /// Player's MongoDB ObjectId (hex string for JSON compat)
    pub player_id: String,

    /// Display name at time of token issuance
    pub name: String,

    /// Issued At (Unix timestamp) - auto-set by jsonwebtoken
    pub iat: i64,

    /// Expiration (Unix timestamp)
    pub exp: i64,
}