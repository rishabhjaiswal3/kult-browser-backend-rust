// src/moments/da_events/model.rs

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

// Event type constants
pub const EVENT_MOMENT_CREATED: &str = "moment.created";
pub const EVENT_MOMENT_LIKED: &str = "moment.liked";
pub const EVENT_MOMENT_COMMENTED: &str = "moment.commented";
pub const EVENT_MOMENT_SHARED: &str = "moment.shared";
pub const EVENT_MOMENT_AI_PROCESSED: &str = "moment.ai_processed";
pub const EVENT_ASSET_MIGRATED: &str = "moment.asset_migrated";
pub const EVENT_COMMENT_CREATED: &str = "comment.created";

// DA status constants — lifecycle of a blob through the 0G DA pipeline
pub const DA_STATUS_PENDING: &str = "pending"; // Created, not yet submitted
pub const DA_STATUS_DISPERSING: &str = "dispersing"; // Submitted to disperser, awaiting finalization
pub const DA_STATUS_FINALIZED: &str = "finalized"; // Finalized with full DA receipt
pub const DA_STATUS_FAILED: &str = "failed"; // Permanently failed

/// DA event record — a verifiable event blob stored on 0G DA.
/// Collection: `momentDaEvents`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentDAEventModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "momentId")]
    pub moment_id: String,

    /// e.g. "moment.created", "moment.liked", "moment.ai_processed"
    #[serde(rename = "eventType")]
    pub event_type: String,

    #[serde(rename = "actorWallet")]
    pub actor_wallet: String,

    // === DA Pipeline Status ===
    /// "pending" | "dispersing" | "finalized" | "failed"
    #[serde(rename = "daStatus")]
    pub da_status: String,

    #[serde(rename = "daError", default)]
    pub da_error: Option<String>,

    // === DA Disperser Receipt (populated after submission) ===
    /// Request ID returned by the 0G DA disperser on submission
    #[serde(rename = "daRequestId", default)]
    pub da_request_id: Option<String>,

    // === DA Finalization Receipt (populated after finalization) ===
    /// 0G DA batch ID — identifies the batch this blob was included in
    #[serde(rename = "daBatchId", default)]
    pub da_batch_id: Option<u64>,

    /// Index of this blob within the batch
    #[serde(rename = "daBlobIndex", default)]
    pub da_blob_index: Option<u32>,

    /// Root hash of the DA batch header — the primary on-chain anchor
    #[serde(rename = "daBatchHeaderHash", default)]
    pub da_batch_header_hash: Option<String>,

    /// L1 block number where the batch was confirmed
    #[serde(rename = "daConfirmationBlock", default)]
    pub da_confirmation_block: Option<u64>,

    /// ISO 8601 timestamp when finalization was detected
    #[serde(rename = "daFinalizedAt", default)]
    pub da_finalized_at: Option<String>,

    /// Event-specific payload included in the DA blob
    #[serde(rename = "eventData")]
    pub event_data: mongodb::bson::Document,

    #[serde(rename = "createdAt", default)]
    pub created_at: Option<DateTime>,

    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<DateTime>,
}

impl MomentDAEventModel {
    pub fn new(
        moment_id: impl Into<String>,
        event_type: impl Into<String>,
        actor_wallet: impl Into<String>,
        event_data: serde_json::Value,
    ) -> Self {
        Self {
            id: None,
            moment_id: moment_id.into(),
            event_type: event_type.into(),
            actor_wallet: actor_wallet.into(),
            da_status: DA_STATUS_PENDING.to_string(),
            da_error: None,
            da_request_id: None,
            da_batch_id: None,
            da_blob_index: None,
            da_batch_header_hash: None,
            da_confirmation_block: None,
            da_finalized_at: None,
            event_data: mongodb::bson::to_document(&event_data).unwrap_or_default(),
            created_at: None,
            updated_at: None,
        }
    }
}
