use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// Comment and reply entity stored in MongoDB.
/// Collection: `moment_comments`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "momentId")]
    pub moment_id: String,

    #[serde(rename = "parentCommentId", default)]
    pub parent_comment_id: Option<ObjectId>,

    #[serde(rename = "authorWalletAddress")]
    pub author_wallet_address: String,

    pub content: String,

    #[serde(rename = "replyCount", default)]
    pub reply_count: u32,

    #[serde(rename = "isEdited", default)]
    pub is_edited: bool,

    #[serde(rename = "isDeleted", default)]
    pub is_deleted: bool,

    #[serde(rename = "deletedAt", default)]
    pub deleted_at: Option<DateTime>,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}
