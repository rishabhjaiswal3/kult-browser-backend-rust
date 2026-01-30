use crate::leaderboard::model::GameLeaderboardConfig;
use futures::stream::TryStreamExt;
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct GameLeaderboardConfigRepository {
    collection: Collection<GameLeaderboardConfig>,
}

impl GameLeaderboardConfigRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("store_games_leaderboards"),
        }
    }

    pub async fn find_all(&self) -> mongodb::error::Result<Vec<GameLeaderboardConfig>> {
        let mut cursor = self.collection.find(mongodb::bson::doc! {}).await?;
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
            .find_one(mongodb::bson::doc! { "identification": identification })
            .await
            .ok()
            .flatten()
    }

    pub async fn upsert(
        &self,
        config: &GameLeaderboardConfig,
    ) -> mongodb::error::Result<mongodb::results::UpdateResult> {
        let filter = mongodb::bson::doc! { "identification": &config.identification };

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
