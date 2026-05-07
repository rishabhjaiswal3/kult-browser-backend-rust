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

    // === 0G Compute AI ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_caption: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_rank_score: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ai_highlights: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_status: Option<String>,

    // === 0G Compute Gameplay Intelligence ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_moment_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_skill_score: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_reaction_quality: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_rarity: Option<String>,

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

/// A single DA event record for the moment timeline.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MomentDAEventResponse {
    pub event_type: String,
    pub actor_wallet: String,
    /// "pending" | "dispersing" | "finalized" | "failed"
    pub da_status: String,
    // DA disperser receipt (populated after finalization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_batch_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_blob_index: Option<u32>,
    /// Root hash of the DA batch — the primary on-chain proof anchor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_batch_header_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_confirmation_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_finalized_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub da_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Full proof bundle for a moment — returned by GET /moments/:id/proof.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MomentProofResponse {
    pub moment_id: String,
    pub storage: StorageProofResponse,
    pub da: DaProofResponse,
    pub compute: ComputeProofResponse,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageProofResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DaProofResponse {
    pub total_events: usize,
    pub finalized_events: usize,
    pub events: Vec<MomentDAEventResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComputeProofResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moment_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
}

/// Pipeline processing status for a moment across all 0G layers.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MomentPipelineResponse {
    pub moment_id: String,
    pub storage: PipelineStageStatus,
    pub da: DaPipelineStatus,
    pub compute: PipelineStageStatus,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStageStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DaPipelineStatus {
    pub total: usize,
    pub finalized: usize,
    pub dispersing: usize,
    pub pending: usize,
    pub failed: usize,
}

/// Response from POST /moments/:id/share
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareMomentResponse {
    pub moment_id: String,
    pub message: String,
}
