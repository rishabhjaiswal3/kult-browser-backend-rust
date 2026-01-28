use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

use super::gameImageModel::GameImages;

// Main game entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameModel {
    // Essential
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub identification: String, // Unique slug (game name with lowercase and hyphens)
    pub name: String,
    pub platform: String, // "web", "desktop", "mobile"
    pub url: String,
    pub images: GameImages,

    // Optional
    pub slogan: Option<String>,
    pub about: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,

    // Extra
    pub metadata: Option<serde_json::Value>,

    // Timestamps
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}