use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

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
    pub field_mappings: Option<Vec<FieldMapping>>, // Renaming and flattening
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub response_key: String, // Output JSON key (e.g. "cover_image")
    pub db_path: String,      // DB path (e.g. "images.hero.horizontal.en.url")
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MappedContentItem {
    /// Flattened content object produced by `field_mappings`.
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
pub enum ContentItem {
    /// Full game document when no field mapping is configured for the section.
    Game(crate::game::model::GameModel),
    /// Field-projected object when the section config remaps the response keys.
    Mapped(MappedContentItem),
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContentResponse {
    #[schema(value_type = Vec<ContentItem>)]
    pub content: Vec<serde_json::Value>, // Generic Value for projection
    pub total_content_count: u32,
    pub page: u32,
    pub page_size: u32,
}
