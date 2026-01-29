use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Game,
    Campaign,  // Future
    Challenge, // Future
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub page: String,    // e.g. "home"
    pub section: String, // e.g. "top_picks"
    pub content_type: ContentType,
    pub content_order: Vec<String>, // List of IDs/Slugs

    #[serde(default)]
    pub content_attributes: Option<Vec<String>>, // ["name", "slogan"] (Projection)
}

#[derive(Debug, Serialize)]
pub struct ContentResponse {
    pub content: Vec<serde_json::Value>, // Generic Value for projection
    pub total_content_count: u32,
    pub page: u32,
    pub page_size: u32,
}
