// src/player/model/update_name_dto.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// PATCH /api/player/name - Request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNameRequest {
    pub name: String,
}

/// PATCH /api/player/name - Response body
#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateNameResponse {
    pub name: String,
}
