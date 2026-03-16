// src/player/model/login_dto.rs

use serde::{Deserialize, Serialize};

/// Payload for registering or logging in.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// The Ethereum wallet address
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,

    /// Optional initial display name
    pub name: Option<String>,

    /// Optional arbitrary metadata (e.g. avatar)
    pub metadata: Option<serde_json::Value>,

    /// Optional referral code provided by the frontend if they clicked an invite link
    #[serde(default)]
    #[serde(rename = "referralCode")]
    pub referral_code: Option<String>,
}

/// POST /api/player/login - Response body
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub player: PlayerInfo,
}

/// Minimal player info returned on login
#[derive(Debug, Serialize)]
pub struct PlayerInfo {
    pub id: String,

    #[serde(rename = "walletAddress")]
    pub wallet_address: String,

    pub name: String,
}
