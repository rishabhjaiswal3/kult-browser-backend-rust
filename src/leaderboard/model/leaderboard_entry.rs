use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player: String, // walletAddress or username
    pub score: f64,     // Normalized score

    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>, // For global leaderboard logic

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>, // Extra fields projected from DB
}
