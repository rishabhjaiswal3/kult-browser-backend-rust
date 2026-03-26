use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ReturnDocument,
    Collection, Database,
};

use super::super::model::{
    platform::Platform,
    post_model::{SharedPost, ValidationStatus},
};
use crate::config::CONFIG;

#[derive(Clone)]
pub struct PostRepository {
    collection: Collection<SharedPost>,
}

impl PostRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.collection::<SharedPost>(&CONFIG.db.shared_posts_collection);
        Self { collection }
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
        let mut cursor = self
            .collection
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .await?;

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

    pub async fn get_post_by_id(
        &self,
        id: ObjectId,
    ) -> Result<Option<SharedPost>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn delete_post(&self, id: ObjectId) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn mark_post_pending(
        &self,
        id: ObjectId,
    ) -> Result<Option<SharedPost>, mongodb::error::Error> {
        let now = chrono::Utc::now();

        self.collection
            .find_one_and_update(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "score": 0,
                        "is_validated": false,
                        "validation_status": "Pending",
                        "validation_reason": mongodb::bson::Bson::Null,
                        "last_validated_at": mongodb::bson::Bson::Null,
                        "updated_at": mongodb::bson::DateTime::from_millis(now.timestamp_millis())
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await
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
