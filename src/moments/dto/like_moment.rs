use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LikeMomentResponse {
    pub moment_id: String,
    pub num_likes: u64,
    pub message: String,
}
