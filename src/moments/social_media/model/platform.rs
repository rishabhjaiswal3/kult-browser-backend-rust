use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub enum Platform {
    Twitter,
    Instagram,
    TikTok,
    Facebook,
    Reddit,
    LinkedIn,
    Pinterest,
    Farcaster,
}
