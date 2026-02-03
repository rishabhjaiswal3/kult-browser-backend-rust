use crate::leaderboard::model::GlobalLeaderboardModel;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct GlobalLeaderboardRepository {
    collection: Collection<GlobalLeaderboardModel>,
}

impl GlobalLeaderboardRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("global_leaderboards"),
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
        // Simple approach: Delete all and insert batch
        // In production, consider using a temporary collection and renaming for atomicity.

        self.collection.delete_many(mongodb::bson::doc! {}).await?;

        if !entries.is_empty() {
            self.collection.insert_many(entries).await?;
        }

        Ok(())
    }

    // Add this method to GlobalLeaderboardRepository impl block:

    /// Get a single player's entry from the global leaderboard.
    pub async fn get_player_entry(
        &self,
        wallet_address: &str,
    ) -> Result<Option<GlobalLeaderboardModel>, String> {
        let normalized = wallet_address.trim().to_lowercase();

        self.collection
            .find_one(doc! { "walletAddress": &normalized })
            .await
            .map_err(|e| format!("Failed to get player entry: {}", e))
    }
}
