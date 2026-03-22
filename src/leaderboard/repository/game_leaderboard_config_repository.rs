use crate::config::CONFIG;
use crate::leaderboard::model::GameLeaderboardConfig;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};

#[derive(Clone)]
pub struct GameLeaderboardConfigRepository {
    collection: Collection<GameLeaderboardConfig>,
}

impl GameLeaderboardConfigRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.game_leaderboard_config_collection),
        }
    }

    pub async fn find_all(&self) -> mongodb::error::Result<Vec<GameLeaderboardConfig>> {
        let mut cursor = self.collection.find(doc! {}).await?;
        let mut configs = Vec::new();
        while let Some(config) = cursor.try_next().await? {
            configs.push(config);
        }
        Ok(configs)
    }

    pub async fn find_by_identification(
        &self,
        identification: &str,
    ) -> Option<GameLeaderboardConfig> {
        self.collection
            .find_one(doc! { "identification": identification })
            .await
            .ok()
            .flatten()
    }

    pub async fn exists_by_identification(
        &self,
        identification: &str,
    ) -> mongodb::error::Result<bool> {
        let count = self
            .collection
            .count_documents(doc! { "identification": identification })
            .await?;

        Ok(count > 0)
    }

    pub async fn insert(&self, config: &GameLeaderboardConfig) -> mongodb::error::Result<ObjectId> {
        let result = self.collection.insert_one(config).await?;
        Ok(result
            .inserted_id
            .as_object_id()
            .unwrap_or_else(ObjectId::new))
    }

    pub async fn delete_by_identification(
        &self,
        identification: &str,
    ) -> mongodb::error::Result<bool> {
        let result = self
            .collection
            .delete_one(doc! { "identification": identification })
            .await?;

        Ok(result.deleted_count > 0)
    }

    pub async fn upsert(
        &self,
        config: &GameLeaderboardConfig,
    ) -> mongodb::error::Result<mongodb::results::UpdateResult> {
        let filter = doc! { "identification": &config.identification };

        self.collection
            .replace_one(filter, config)
            .with_options(
                mongodb::options::ReplaceOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await
    }
}
