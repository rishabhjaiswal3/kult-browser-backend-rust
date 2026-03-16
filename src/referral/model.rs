// src/referral/model.rs

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Represents a successful, validated referral connection between two players.
/// Collection: `store_referrals`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralConnection {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The string ID (e.g., wallet address or hex ObjectId) of the player who owns the link
    pub referrer_id: String,

    /// The string ID (e.g., wallet address or hex ObjectId) of the newly joined player
    pub referred_player_id: String,

    /// The 6-8 character nanoID code that was used (e.g. "x7B9qz")
    pub code_used: String,

    /// The amount of KULT points or virtual currency granted for this referral
    pub reward_issued: u32,

    pub created_at: DateTime<Utc>,
}
