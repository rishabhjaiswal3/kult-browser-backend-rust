use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLeaderboardConfig {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub identification: String, // e.g., "guesstheai"
    pub db: String,             // Target Database Name
    pub collection: String,     // Target Collection Name

    #[serde(rename = "scoreKey")]
    pub score_key: String, // e.g., "score.total" or "highScore"

    #[serde(rename = "personKey")]
    pub person_key: String, // e.g., "walletAddress"

    pub order: i32, // 1 (ASC) or -1 (DESC)

    #[serde(default = "default_weight")]
    pub weight: f64, // Score Multiplier for Global Leaderboard

    #[serde(default)]
    pub projection: Option<Vec<String>>,
}

fn default_weight() -> f64 {
    1.0
}
