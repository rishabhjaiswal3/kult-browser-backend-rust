use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponse {
    pub comment_id: String,
    pub moment_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_comment_id: Option<String>,

    pub author_wallet_address: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    pub reply_count: u32,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteCommentResponse {
    pub message: String,
}
