use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequeueSharedPostResponse {
    pub post_id: String,
    pub message: String,
}
