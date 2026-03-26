use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatorProfileResponse {
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub rank: u64,
    pub total_moments: u64,
    pub total_moment_likes: u64,
    pub total_moment_comments: u64,
    pub total_social_likes: u64,
    pub validated_posts_count: u64,
    pub successful_referrals: u64,
    pub total_score: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatorLeaderboardEntry {
    pub rank: u64,
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub total_moments: u64,
    pub total_moment_likes: u64,
    pub total_moment_comments: u64,
    pub total_social_likes: u64,
    pub validated_posts_count: u64,
    pub successful_referrals: u64,
    pub total_score: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatorLeaderboardResponse {
    pub entries: Vec<CreatorLeaderboardEntry>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}
