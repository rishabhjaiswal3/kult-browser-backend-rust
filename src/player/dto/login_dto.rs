// src/player/model/login_dto.rs

use serde::{Deserialize, Serialize};

/// POST /api/player/login - Request body
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Required: Ethereum wallet address
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,

    /// Optional: Display name (auto-generated if not provided)
    pub name: Option<String>,

    /// Optional: Arbitrary metadata to store
    pub metadata: Option<serde_json::Value>,
}

/// POST /api/player/login - Response body
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub ok: bool,
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