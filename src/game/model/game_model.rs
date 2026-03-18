use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::game_image_model::GameImages;

// Main game entity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameModel {
    // Essential
    #[serde(rename = "_id")]
    #[schema(value_type = String)]
    pub id: ObjectId,
    pub identification: String, // Unique slug (game name with lowercase and hyphens)
    pub name: super::Localized<String>,
    pub platform: String, // "web", "desktop", "mobile"
    pub url: String,
    pub images: GameImages,

    // Optional
    pub slogan: Option<super::Localized<String>>,
    pub about: Option<super::Localized<String>>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,

    // Extra
    #[schema(value_type = Option<Object>)]
    pub metadata: Option<serde_json::Value>,

    // Timestamps
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
