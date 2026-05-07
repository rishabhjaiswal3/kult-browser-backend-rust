use mongodb::bson::{doc, DateTime as BsonDateTime};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerNonce {
    pub nonce: String,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(rename = "createdAt")]
    pub created_at: BsonDateTime,
}

#[derive(Clone)]
pub struct NonceRepository {
    collection: Collection<PlayerNonce>,
}

impl NonceRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.player_nonces_collection),
        }
    }

    pub async fn create_nonce(
        &self,
        wallet: &str,
        nonce: &str,
    ) -> Result<(), mongodb::error::Error> {
        // Remove any existing nonce for this wallet before issuing a new one
        let _ = self
            .collection
            .delete_many(doc! { "walletAddress": wallet })
            .await;

        let doc = PlayerNonce {
            nonce: nonce.to_string(),
            wallet_address: wallet.to_string(),
            created_at: BsonDateTime::now(),
        };
        self.collection.insert_one(doc).await?;
        Ok(())
    }

    /// Delete the nonce if it exists for this wallet. Returns true if it was found and deleted.
    pub async fn consume_nonce(
        &self,
        wallet: &str,
        nonce: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .delete_one(doc! { "walletAddress": wallet, "nonce": nonce })
            .await?;
        Ok(result.deleted_count > 0)
    }
}
