// src/marketplace/orders/dto/order_response.rs

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Request payload for purchasing a listing.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    /// Listing ID to purchase
    pub listing_id: String,

    /// Quantity to purchase (default 1)
    #[serde(default = "default_quantity")]
    pub quantity: u32,

    /// On-chain transaction hash (optional)
    pub tx_hash: Option<String>,
}

fn default_quantity() -> u32 {
    1
}

/// Single order response for API output.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub id: String,
    pub listing_id: String,
    pub player_id: String,
    pub game_identification: String,
    pub price_paid: f64,
    pub quantity: u32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
}

/// Paginated list of orders.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderListResponse {
    pub orders: Vec<OrderResponse>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

/// Query parameters for order listing.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct OrdersQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
