// src/leaderboard/dto/leaderboard_dto.rs

use serde::{Deserialize, Serialize};

use crate::leaderboard::model::{GlobalLeaderboardModel, LeaderboardEntry};

/// Paginated response for global leaderboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLeaderboardResponse {
    pub entries: Vec<GlobalLeaderboardEntryDto>,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "totalPages")]
    pub total_pages: u32,
}

/// DTO for a single global leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLeaderboardEntryDto {
    pub rank: u32,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    pub score: f64,
    pub level: u32,
}

impl From<GlobalLeaderboardModel> for GlobalLeaderboardEntryDto {
    fn from(model: GlobalLeaderboardModel) -> Self {
        Self {
            rank: model.rank,
            wallet_address: model.wallet_address,
            score: model.score,
            level: model.level,
        }
    }
}

/// Paginated response for game-specific leaderboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLeaderboardResponse {
    pub entries: Vec<GameLeaderboardEntryDto>,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "totalPages")]
    pub total_pages: u32,
}

/// DTO for a single game leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLeaderboardEntryDto {
    pub rank: u32,
    pub player: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl GameLeaderboardEntryDto {
    pub fn from_entry(entry: LeaderboardEntry, rank: u32) -> Self {
        Self {
            rank,
            player: entry.player,
            score: entry.score,
            level: entry.level,
            metadata: entry.metadata,
        }
    }
}
