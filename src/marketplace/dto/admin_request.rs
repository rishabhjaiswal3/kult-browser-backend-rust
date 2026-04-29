// src/marketplace/dto/admin_request.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request payload for creating a marketplace listing (admin).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateListingRequest {
    pub name: String,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub asset_url: Option<String>,
    pub price: f64,
    pub category: String,
    pub currency: String,
    pub game_identification: String,
    pub contract_item_id: Option<String>,
}

/// Request payload for updating a marketplace listing (admin).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListingRequest {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub asset_url: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub currency: Option<String>,
    pub contract_item_id: Option<String>,
}

/// Response for delist operation.
#[derive(Debug, Serialize, ToSchema)]
pub struct DelistResponse {
    pub id: String,
    pub message: String,
}
