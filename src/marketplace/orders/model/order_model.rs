// src/marketplace/orders/model/order_model.rs

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Marketplace order entity stored in MongoDB.
/// Collection: `marketplace_orders`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Reference to the marketplace listing
    #[serde(rename = "listingId")]
    pub listing_id: ObjectId,

    /// Backend-generated order identifier (used to bind pre/post-payment steps).
    #[serde(rename = "orderId")]
    pub order_id: String,

    /// Buyer's player ID (ObjectId as string from auth)
    #[serde(rename = "playerId")]
    pub player_id: String,

    /// Buyer's wallet address at prepare time.
    #[serde(rename = "buyerWallet")]
    pub buyer_wallet: String,

    /// Denormalized game identification slug for easy queries
    #[serde(rename = "gameIdentification")]
    pub game_identification: String,

    /// Payment token address selected for this order.
    #[serde(rename = "paymentToken")]
    pub payment_token: String,

    /// Price snapshot at purchase time
    #[serde(rename = "pricePaid")]
    pub price_paid: f64,

    /// Quantity purchased
    #[serde(default = "default_quantity")]
    pub quantity: u32,

    /// Order status: pending, completed, failed, refunded
    #[serde(default = "default_order_status")]
    pub status: String,

    /// On-chain transaction hash (if applicable)
    #[serde(rename = "txHash", default)]
    pub tx_hash: Option<String>,

    #[serde(rename = "createdAt", default)]
    pub created_at: Option<mongodb::bson::DateTime>,
}

fn default_quantity() -> u32 {
    1
}

fn default_order_status() -> String {
    "pending".to_string()
}
