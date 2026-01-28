use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::FindOneAndUpdateOptions, Collection, Database};
use std::env;

use crate::GameModel;

pub struct GameModelRepository {
    collection: Collection<GameModel>,
}

impl GameModelRepository {
    pub fn new(db: &Database) -> Self {
        dotenvy::dotenv().ok();

        let games_mongo_coll_name =
            env::var("GAMES_MONGO_COLL_NAME").unwrap_or_else(|_| "kultbrowser_games".to_string());
        Self {
            collection: db.collection::<GameModel>(&games_mongo_coll_name),
        }
    }

    // Find a GameModel by its unique slug
    pub async fn find_by_identification(
        &self,
        identification: &str,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        self.collection
            .find_one(doc! { "identification": identification })
            .await
    }
    // Check if a GameModel exists
    pub async fn exists(&self, identification: &str) -> Result<bool, mongodb::error::Error> {
        let count = self
            .collection
            .count_documents(doc! { "identification": identification })
            .await?;
        Ok(count > 0)
    }
    // Create a new GameModel
    pub async fn create(&self, game_model: &GameModel) -> Result<String, mongodb::error::Error> {
        let result = self.collection.insert_one(game_model).await?;
        Ok(result.inserted_id.as_object_id().unwrap().to_hex())
    }
    // Patch (partial update) - only updates provided fields
    pub async fn patch(
        &self,
        identification: &str,
        updates: mongodb::bson::Document,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let mut update_doc = doc! {
            "$set": {
                "updated_at": Utc::now().to_rfc3339()
            }
        };
        // Merge the provided updates into $set
        if let Some(set_doc) = updates.get("$set") {
            if let Some(set_obj) = set_doc.as_document() {
                for (key, value) in set_obj {
                    update_doc
                        .get_document_mut("$set")
                        .unwrap()
                        .insert(key.clone(), value.clone());
                }
            }
        }
        // Handle $unset if present
        if let Some(unset_doc) = updates.get("$unset") {
            update_doc.insert("$unset", unset_doc.clone());
        }
        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();
        self.collection
            .find_one_and_update(doc! { "identification": identification }, update_doc)
            .with_options(options)
            .await
    }
    // Replace (full update) - replaces entire document
    pub async fn replace(
        &self,
        identification: &str,
        game_model: &GameModel,
    ) -> Result<Option<GameModel>, mongodb::error::Error> {
        let options = mongodb::options::FindOneAndReplaceOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .upsert(true)
            .build();
        self.collection
            .find_one_and_replace(doc! { "identification": identification }, game_model)
            .with_options(options)
            .await
    }
    // Delete a GameModel
    pub async fn delete(&self, identification: &str) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .delete_one(doc! { "identification": identification })
            .await?;
        Ok(result.deleted_count > 0)
    }
    // Find all GameModels (with optional limit)
    pub async fn find_all(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<GameModel>, mongodb::error::Error> {
        let mut cursor = self.collection.find(doc! {}).await?;
        let mut game_models = Vec::new();
        let max = limit.unwrap_or(100) as usize;
        while let Some(game_model) = cursor.try_next().await? {
            game_models.push(game_model);
            if game_models.len() >= max {
                break;
            }
        }
        Ok(game_models)
    }
}
