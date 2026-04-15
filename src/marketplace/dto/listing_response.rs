// src/marketplace/dto/listing_response.rs

use serde::Serialize;
use utoipa::ToSchema;

/// Single listing response for API output.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListingResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub asset_type: String,
    pub game_identification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u64>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub attributes: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated list of listings.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListingListResponse {
    pub listings: Vec<ListingResponse>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}
