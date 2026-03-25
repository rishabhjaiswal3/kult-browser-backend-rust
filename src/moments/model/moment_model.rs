// src/moments/model/moment_model.rs

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Moment entity stored in MongoDB.
/// Collection: `moments`
///
/// Represents an image/GIF shared by a player with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Unique shareable ID (NanoID, 21 chars)
    #[serde(rename = "momentId")]
    pub moment_id: String,

    /// Owner's wallet address (lowercase, trimmed)
    #[serde(rename = "playerWalletAddress")]
    pub player_wallet_address: String,

    /// URL to the image/GIF asset (optional — only if uploaded)
    #[serde(rename = "assetUrl", default)]
    pub asset_url: Option<String>,

    /// Original filename from upload
    #[serde(
        rename = "originalFilename",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub original_filename: Option<String>,

    /// File size in bytes from upload
    #[serde(
        rename = "fileSizeBytes",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub file_size_bytes: Option<u64>,

    /// ZG hash of the asset (empty until migrated)
    #[serde(rename = "assetZgHash", default)]
    pub asset_zg_hash: Option<String>,

    /// Total likes on the moment
    #[serde(rename = "numLikes", default)]
    pub num_likes: u64,

    /// Total active comments on the moment, including replies
    #[serde(rename = "numComments", default)]
    pub num_comments: u64,

    /// Optional metadata about the asset (type, size, dimensions, etc.)
    #[serde(rename = "assetMetadata", default)]
    pub asset_metadata: Option<mongodb::bson::Document>,

    /// Title of the moment
    pub title: String,

    /// Description of the moment
    #[serde(default)]
    pub description: Option<String>,

    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Social media links (twitter, instagram, etc.)
    #[serde(rename = "socialMediaLinks", default)]
    pub social_media_links: Option<mongodb::bson::Document>,

    /// Timestamps
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<mongodb::bson::DateTime>,

    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<mongodb::bson::DateTime>,
}

impl MomentModel {
    /// Generate a new NanoID for moment_id
    pub fn generate_moment_id() -> String {
        nanoid::nanoid!()
    }
}
