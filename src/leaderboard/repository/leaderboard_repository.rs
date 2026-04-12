use crate::config::CONFIG;
use crate::leaderboard::model::GlobalLeaderboardModel;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::ReplaceOptions;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct GlobalLeaderboardRepository {
    collection: Collection<GlobalLeaderboardModel>,
}

impl GlobalLeaderboardRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.global_leaderboard_collection),
        }
    }

    pub async fn get_global_ranking(
        &self,
        skip: u64,
        limit: i64,
    ) -> mongodb::error::Result<Vec<GlobalLeaderboardModel>> {
        let options = mongodb::options::FindOptions::builder()
            .skip(skip)
            .limit(limit)
            .sort(mongodb::bson::doc! { "score": -1 }) // Sort by score DESC
            .build();

        let mut cursor = self
            .collection
            .find(mongodb::bson::doc! {})
            .with_options(options)
            .await?;
        let mut entries = Vec::new();
        while let Some(entry) = cursor.try_next().await? {
            entries.push(entry);
        }
        Ok(entries)
    }

    pub async fn replace_all(
        &self,
        entries: &[GlobalLeaderboardModel],
    ) -> mongodb::error::Result<()> {
        if entries.is_empty() {
            self.collection.delete_many(doc! {}).await?;
            return Ok(());
        }

        // Collect all wallet addresses that should remain in the leaderboard
        let active_wallets: Vec<&str> = entries.iter().map(|e| e.wallet_address.as_str()).collect();

        // Bulk upsert: replace each entry atomically so readers never see empty data
        let upsert_opts = ReplaceOptions::builder().upsert(true).build();
        for entry in entries {
            self.collection
                .replace_one(doc! { "walletAddress": &entry.wallet_address }, entry)
                .with_options(upsert_opts.clone())
                .await?;
        }

        // Remove players who are no longer ranked
        self.collection
            .delete_many(doc! { "walletAddress": { "$nin": &active_wallets } })
            .await?;

        Ok(())
    }

    // Add this method to GlobalLeaderboardRepository impl block:

    /// Get a single player's entry from the global leaderboard.
    pub async fn get_player_entry(
        &self,
        wallet_address: &str,
    ) -> Result<Option<GlobalLeaderboardModel>, String> {
        let wallet = wallet_address.trim();

        self.collection
            .find_one(doc! { "walletAddress": wallet })
            .await
            .map_err(|e| format!("Failed to get player entry: {}", e))
    }

    /// Get total count of entries in global leaderboard.
    pub async fn count_all(&self) -> mongodb::error::Result<u64> {
        self.collection.count_documents(doc! {}).await
    }
}
