// src/moments/dto/moment_response.rs

use serde::Serialize;
use utoipa::ToSchema;

/// Single moment response for API output.
#[derive(Debug, Serialize, ToSchema)]
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

    /// 0G metadata hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_zg_hash: Option<String>,

    /// Latest 0G storage status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zg_status: Option<String>,

    /// 0G transaction hash for the asset upload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_zg_tx_hash: Option<String>,

    /// Public 0G gateway URL for the asset, if configured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_zg_url: Option<String>,

    /// 0G transaction hash for the metadata upload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_zg_tx_hash: Option<String>,

    /// Public 0G gateway URL for metadata JSON, if configured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_zg_url: Option<String>,

    /// Explorer URL for the asset upload tx, if configured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_zg_tx_url: Option<String>,

    /// Explorer URL for the metadata upload tx, if configured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_zg_tx_url: Option<String>,

    /// Last 0G migration error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zg_error: Option<String>,

    /// 0G upload completion timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zg_uploaded_at: Option<String>,

    /// Total likes on the moment
    pub num_likes: u64,

    /// Total active comments on the moment, including replies
    pub num_comments: u64,

    /// Asset metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
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

    /// Related game identification slugs
    pub related_games: Vec<String>,

    /// Social media links
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub social_media_links: Option<serde_json::Value>,

    /// Created timestamp (ISO 8601)
    pub created_at: String,

    /// Updated timestamp (ISO 8601)
    pub updated_at: String,
}

/// Paginated list of moments for feed and player's moments.
#[derive(Debug, Serialize, ToSchema)]
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

/// Public 0G proof document for a moment.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MomentZgProofResponse {
    pub moment_id: String,
    pub zg_status: Option<String>,
    pub asset_zg_hash: Option<String>,
    pub metadata_zg_hash: Option<String>,
    pub asset_zg_tx_hash: Option<String>,
    pub metadata_zg_tx_hash: Option<String>,
    pub asset_zg_url: Option<String>,
    pub metadata_zg_url: Option<String>,
    pub asset_zg_tx_url: Option<String>,
    pub metadata_zg_tx_url: Option<String>,
    pub zg_uploaded_at: Option<String>,
    pub zg_error: Option<String>,
}

/// Response after retrying a failed or pending 0G migration.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryZgMigrationResponse {
    pub moment_id: String,
    pub zg_status: String,
    pub message: String,
}
