// src/moments/dto/create_moment.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request payload for creating a new moment.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMomentRequest {
    /// URL to the image/GIF asset (optional — only if already uploaded to DO)
    pub asset_url: Option<String>,

    /// Optional metadata about the asset
    #[schema(value_type = Option<Object>)]
    pub asset_metadata: Option<serde_json::Value>,

    /// Title of the moment (required)
    pub title: String,

    /// Description of the moment (optional)
    pub description: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Related game identification slugs
    #[serde(default)]
    pub related_games: Vec<String>,

    /// Social media links
    #[schema(value_type = Option<Object>)]
    pub social_media_links: Option<serde_json::Value>,
}

/// Response after successfully creating a moment.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMomentResponse {
    /// The generated shareable moment ID
    pub moment_id: String,

    /// Success message
    pub message: String,
}
