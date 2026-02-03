// src/player/model/update_name_dto.rs

use serde::{Deserialize, Serialize};

/// PATCH /api/player/name - Request body
#[derive(Debug, Deserialize)]
pub struct UpdateNameRequest {
    pub name: String,
}

/// PATCH /api/player/name - Response body
#[derive(Debug, Serialize)]
pub struct UpdateNameResponse {
    pub ok: bool,
    pub name: String,
}