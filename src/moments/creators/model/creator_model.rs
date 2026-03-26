use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreatorAggregate {
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(rename = "totalMoments")]
    pub total_moments: i64,
    #[serde(rename = "totalMomentLikes")]
    pub total_moment_likes: i64,
    #[serde(rename = "totalMomentComments")]
    pub total_moment_comments: i64,
    #[serde(rename = "totalSocialLikes")]
    pub total_social_likes: i64,
    #[serde(rename = "validatedPostsCount")]
    pub validated_posts_count: i64,
    #[serde(rename = "successfulReferrals")]
    pub successful_referrals: i64,
    #[serde(rename = "totalScore")]
    pub total_score: i64,
}
