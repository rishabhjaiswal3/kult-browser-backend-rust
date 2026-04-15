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
    #[serde(default)]
    pub description: Option<String>,

    /// Asset type: weapon, skin, power_up, collectible, etc.
    #[serde(rename = "assetType")]
    pub asset_type: String,

    /// Game identification slug (stable reference, survives re-ingestion)
    #[serde(rename = "gameIdentification")]
    pub game_identification: String,

    /// Thumbnail image URL
    #[serde(rename = "thumbnailUrl", default)]
    pub thumbnail_url: Option<String>,

    /// Price in A0GI
    pub price: f64,

    /// Total supply (null = unlimited)
    #[serde(default)]
    pub supply: Option<u64>,

    /// Remaining supply (null = unlimited)
    #[serde(default)]
    pub remaining: Option<u64>,

    /// Listing status: active, sold_out, delisted
    #[serde(default = "default_status")]
    pub status: String,

    /// Flexible game-specific metadata
    #[serde(default)]
    pub attributes: Option<mongodb::bson::Document>,

    #[serde(rename = "createdAt", default)]
    pub created_at: Option<mongodb::bson::DateTime>,

    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<mongodb::bson::DateTime>,
}

fn default_status() -> String {
    "active".to_string()
}
