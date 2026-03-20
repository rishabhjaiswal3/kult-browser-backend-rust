use super::platform::Platform;
use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedPost {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // Core Relations
    pub moment_id: Bson,
    pub wallet_address: String,

    // Social Media Data
    pub platform: Platform,
    pub post_id: String,
    pub url: String,

    // Engagement / Scraper Data
    pub num_likes: u32,
    pub score: u32,

    // Validation State
    pub is_validated: bool,
    pub validation_status: ValidationStatus,
    pub validation_reason: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::bson_date_option"
    )]
    pub last_validated_at: Option<DateTime<Utc>>,

    // Audit
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
