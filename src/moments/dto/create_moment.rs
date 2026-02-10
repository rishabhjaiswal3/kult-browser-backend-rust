// src/moments/dto/create_moment.rs

use serde::{Deserialize, Serialize};

/// Request payload for creating a new moment.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMomentRequest {
    /// URL to the image/GIF asset (optional — only if already uploaded to DO)
    pub asset_url: Option<String>,

    /// Optional metadata about the asset
    pub asset_metadata: Option<serde_json::Value>,

    /// Title of the moment (required)
    pub title: String,

    /// Description of the moment (optional)
    pub description: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Social media links
    pub social_media_links: Option<serde_json::Value>,
}

/// Response after successfully creating a moment.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMomentResponse {
    /// The generated shareable moment ID
    pub moment_id: String,

    /// Success message
    pub message: String,
}
