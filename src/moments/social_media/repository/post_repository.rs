use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};

use super::super::model::{
    platform::Platform,
    post_model::{SharedPost, ValidationStatus},
};
use crate::config::db_config::DbConfig;
use crate::mongo::connection::connect;

#[derive(Clone)]
pub struct PostRepository {
    collection: Collection<SharedPost>,
}

impl PostRepository {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        let db: mongodb::Database = connect().await?;
        // The collection will be strictly named `shared_posts` in MongoDB
        let config = DbConfig::from_env();
        let collection = db.collection::<SharedPost>(&config.shared_posts_collection);
        Ok(Self { collection })
    }

    /// Inserts a new shared post into the database.
    pub async fn create_post(&self, post: SharedPost) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(post).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// Fetches all shared posts explicitly belonging to a wallet address.
    pub async fn get_posts_by_wallet_address(
        &self,
        wallet: &str,
    ) -> Result<Vec<SharedPost>, mongodb::error::Error> {
        let filter = doc! { "wallet_address": wallet };
        let mut cursor = self.collection.find(filter).await?;

        let mut posts = Vec::new();
        while cursor.advance().await? {
            posts.push(cursor.deserialize_current()?);
        }

        Ok(posts)
    }

    /// Checks if a post_id for a certain platform already exists to prevent duplication.
    pub async fn get_post_by_platform_and_id(
        &self,
        platform: Platform,
        post_id: &str,
    ) -> Result<Option<SharedPost>, mongodb::error::Error> {
        // We serialize platform to string for querying since it's an enum
        let platform_str = serde_json::to_string(&platform)
            .unwrap_or_default()
            .replace("\"", "");

        let filter = doc! {
            "platform": platform_str,
            "post_id": post_id
        };
        self.collection.find_one(filter).await
    }

    /// Updates the engagement metrics of a post and its validation status. (Called by the Scraper).
    pub async fn update_post_metrics(
        &self,
        id: ObjectId,
        likes: u32,
        score: u32,
        is_validated: bool,
        validation_status: ValidationStatus,
        validation_reason: &str,
    ) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "_id": id };

        let status_str = serde_json::to_string(&validation_status)
            .unwrap_or_default()
            .replace("\"", "");
        let now = chrono::Utc::now();

        // Use MongoDB $set to update specific fields without wiping the rest
        let update = doc! {
            "$set": {
                "num_likes": likes,
                "score": score,
                "is_validated": is_validated,
                "validation_status": status_str,
                "validation_reason": validation_reason,
                "last_validated_at": mongodb::bson::DateTime::from_millis(now.timestamp_millis()),
                "updated_at": mongodb::bson::DateTime::from_millis(now.timestamp_millis())
            }
        };

        self.collection.update_one(filter, update).await?;
        Ok(())
    }
}
