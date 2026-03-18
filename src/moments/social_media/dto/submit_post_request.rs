// src/moments/social_media/dto/submit_post_request.rs

use serde::Deserialize;
use utoipa::ToSchema;

use crate::moments::social_media::model::platform::Platform;

/// Request body for POST /api/moments/social-media/submit-url
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitPostRequest {
    /// The ID of the moment being shared
    pub moment_id: String,
    /// The social media platform
    pub platform: Platform,
    /// The unique post ID on that platform (e.g. tweet ID)
    pub post_id: String,
    /// The full URL to the social media post
    pub url: String,
}
