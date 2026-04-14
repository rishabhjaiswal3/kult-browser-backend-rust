// src/moments/dto/update_moment.rs

use serde::Deserialize;
use utoipa::ToSchema;

/// Request payload for updating an existing moment.
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMomentRequest {
    /// Updated asset URL
    pub asset_url: Option<String>,

    /// Updated asset metadata
    #[schema(value_type = Option<Object>)]
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

    /// Updated related game identification slugs
    pub related_games: Option<Vec<String>>,

    /// Updated social media links
    #[schema(value_type = Option<Object>)]
    pub social_media_links: Option<serde_json::Value>,
}
