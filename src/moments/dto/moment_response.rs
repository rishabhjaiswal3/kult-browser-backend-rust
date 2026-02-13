// src/moments/dto/moment_response.rs

use serde::Serialize;

/// Single moment response for API output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentResponse {
    /// Shareable moment ID
    pub moment_id: String,

    /// Owner's wallet address
    pub player_wallet_address: String,

    /// Asset URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_url: Option<String>,

    /// 0G storage hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_zg_hash: Option<String>,

    /// Asset metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_metadata: Option<serde_json::Value>,

    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,

    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<u64>,

    /// Title
    pub title: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tags
    pub tags: Vec<String>,

    /// Social media links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media_links: Option<serde_json::Value>,

    /// Created timestamp (ISO 8601)
    pub created_at: String,

    /// Updated timestamp (ISO 8601)
    pub updated_at: String,
}

/// Paginated list of moments for feed and player's moments.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentListResponse {
    /// List of moments
    pub moments: Vec<MomentResponse>,

    /// Total count of moments matching the query
    pub total: u64,

    /// Current page (1-indexed)
    pub page: u32,

    /// Number of items per page
    pub per_page: u32,
}
