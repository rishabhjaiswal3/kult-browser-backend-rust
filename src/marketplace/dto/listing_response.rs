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
    pub short_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_url: Option<String>,
    pub price: f64,
    pub category: String,
    pub currency: String,
    pub game_identification: String,
    pub status: String,
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
