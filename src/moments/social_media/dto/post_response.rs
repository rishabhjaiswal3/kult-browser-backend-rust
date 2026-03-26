use serde::Serialize;
use utoipa::ToSchema;

use crate::moments::social_media::model::{platform::Platform, post_model::ValidationStatus};

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedPostResponse {
    pub id: String,
    pub moment_id: String,
    pub platform: Platform,
    pub external_post_id: String,
    pub url: String,
    pub num_likes: u32,
    pub score: u32,
    pub is_validated: bool,
    pub validation_status: ValidationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedPostListResponse {
    pub posts: Vec<SharedPostResponse>,
}
