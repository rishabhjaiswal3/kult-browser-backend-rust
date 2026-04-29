// src/marketplace/model/listing_model.rs

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Marketplace listing entity stored in MongoDB.
/// Collection: `marketplace_listings`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Display name of the asset
    pub name: String,

    /// Short description
    #[serde(rename = "shortDescription", default)]
    pub short_description: Option<String>,

    /// Long description
    #[serde(rename = "longDescription", default)]
    pub long_description: Option<String>,

    /// Asset URL (image / media)
    #[serde(rename = "assetUrl", default)]
    pub asset_url: Option<String>,

    /// Price in the listing's currency
    pub price: f64,

    /// Category: Coins, Gems, Guns, etc.
    pub category: String,

    /// Currency: SOMI, ETH, etc.
    pub currency: String,

    /// Game identification slug (stable reference, survives re-ingestion)
    #[serde(rename = "gameIdentification")]
    pub game_identification: String,

    /// Contract item ID used by UnifiedGameMarketplace.purchase itemId argument.
    #[serde(rename = "contractItemId", default)]
    pub contract_item_id: Option<String>,

    /// Listing status: active, delisted
    #[serde(default = "default_status")]
    pub status: String,

    #[serde(rename = "createdAt", default)]
    pub created_at: Option<mongodb::bson::DateTime>,

    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<mongodb::bson::DateTime>,
}

fn default_status() -> String {
    "active".to_string()
}
