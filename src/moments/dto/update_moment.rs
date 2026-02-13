// src/moments/dto/update_moment.rs

use serde::Deserialize;

/// Request payload for updating an existing moment.
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMomentRequest {
    /// Updated asset URL
    pub asset_url: Option<String>,

    /// Updated asset metadata
    pub asset_metadata: Option<serde_json::Value>,

    /// Original filename from upload
    pub original_filename: Option<String>,

    /// File size in bytes from upload
    pub file_size_bytes: Option<u64>,

    /// Updated title
    pub title: Option<String>,

    /// Updated description
    pub description: Option<String>,

    /// Updated tags
    pub tags: Option<Vec<String>>,

    /// Updated social media links
    pub social_media_links: Option<serde_json::Value>,
}
