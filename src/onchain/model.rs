use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    MomentCreated,
    MomentUpdated,
    MomentDeleted,
    MomentLiked,
    CommentCreated,
    CommentDeleted,
    ReplyCreated,
    SocialPostSubmitted,
    SocialPostValidated,
    AssetMigratedTo0g,
}

impl ActivityType {
    pub fn contract_value(self) -> u8 {
        match self {
            Self::MomentCreated => 0,
            Self::MomentUpdated => 1,
            Self::MomentDeleted => 2,
            Self::MomentLiked => 3,
            Self::CommentCreated => 4,
            Self::CommentDeleted => 5,
            Self::ReplyCreated => 6,
            Self::SocialPostSubmitted => 7,
            Self::SocialPostValidated => 8,
            Self::AssetMigratedTo0g => 9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnchainActivityStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnchainActivityJob {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "activityId")]
    pub activity_id: String,

    #[serde(rename = "userWallet")]
    pub user_wallet: String,

    #[serde(rename = "activityType")]
    pub activity_type: ActivityType,

    #[serde(rename = "momentId")]
    pub moment_id: String,

    #[serde(rename = "entityId")]
    pub entity_id: String,

    #[serde(rename = "metadataHash")]
    pub metadata_hash: String,

    #[serde(rename = "status")]
    pub status: OnchainActivityStatus,

    #[serde(rename = "txHash", skip_serializing_if = "Option::is_none", default)]
    pub tx_hash: Option<String>,

    #[serde(default)]
    pub attempts: u32,

    #[serde(rename = "lastError", skip_serializing_if = "Option::is_none", default)]
    pub last_error: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}
