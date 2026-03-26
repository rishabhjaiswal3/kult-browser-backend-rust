use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentLikeModel {
    #[serde(rename = "_id")]
    pub id: String,
    pub moment_id: String,
    pub author_wallet_address: String,
    pub created_at: DateTime,
}

impl MomentLikeModel {
    pub fn new(moment_id: &str, wallet: &str) -> Self {
        let normalized_wallet = wallet.trim().to_lowercase();
        Self {
            id: format!("{moment_id}:{normalized_wallet}"),
            moment_id: moment_id.to_string(),
            author_wallet_address: normalized_wallet,
            created_at: DateTime::now(),
        }
    }
}
