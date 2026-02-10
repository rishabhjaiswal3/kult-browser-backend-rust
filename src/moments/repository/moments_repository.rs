// src/moments/repository/moments_repository.rs

use futures::TryStreamExt;
use mongodb::bson::{doc, DateTime, Document};
use mongodb::{Collection, Database};

use crate::moments::model::MomentModel;

/// Repository for moment database operations.
#[derive(Clone)]
pub struct MomentsRepository {
    collection: Collection<MomentModel>,
    raw_collection: Collection<Document>,
}

impl MomentsRepository {
    /// Create a new repository instance.
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("moments"),
            raw_collection: db.collection("moments"),
        }
    }

    /// Create a new moment.
    pub async fn create(&self, moment: MomentModel) -> Result<String, String> {
        let moment_id = moment.moment_id.clone();

        let mut doc = mongodb::bson::to_document(&moment).map_err(|e| e.to_string())?;
        let now = DateTime::now();
        doc.insert("createdAt", now);
        doc.insert("updatedAt", now);

        self.raw_collection
            .insert_one(doc)
            .await
            .map_err(|e| e.to_string())?;

        Ok(moment_id)
    }

    /// Find a moment by its shareable moment_id.
    pub async fn find_by_moment_id(&self, moment_id: &str) -> Result<Option<MomentModel>, String> {
        self.collection
            .find_one(doc! { "momentId": moment_id })
            .await
            .map_err(|e| e.to_string())
    }

    /// Find moments by player wallet address (paginated).
    pub async fn find_by_wallet(
        &self,
        wallet: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<MomentModel>, String> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;

        let cursor = self
            .collection
            .find(doc! { "playerWalletAddress": wallet })
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(per_page as i64)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Find all moments (public feed, paginated).
    /// Optionally filter by tags (moments must have at least one matching tag).
    pub async fn find_all(
        &self,
        page: u32,
        per_page: u32,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<MomentModel>, String> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;

        let filter = match tags {
            Some(t) if !t.is_empty() => doc! { "tags": { "$in": t } },
            _ => doc! {},
        };

        let cursor = self
            .collection
            .find(filter)
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(per_page as i64)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Count moments by player wallet.
    pub async fn count_by_wallet(&self, wallet: &str) -> Result<u64, String> {
        self.collection
            .count_documents(doc! { "playerWalletAddress": wallet })
            .await
            .map_err(|e| e.to_string())
    }

    /// Count all moments (optionally filtered by tags).
    pub async fn count_all(&self, tags: Option<Vec<String>>) -> Result<u64, String> {
        let filter = match tags {
            Some(t) if !t.is_empty() => doc! { "tags": { "$in": t } },
            _ => doc! {},
        };

        self.collection
            .count_documents(filter)
            .await
            .map_err(|e| e.to_string())
    }

    /// Update a moment by moment_id.
    /// Returns the updated moment if found.
    pub async fn update(
        &self,
        moment_id: &str,
        updates: Document,
    ) -> Result<Option<MomentModel>, String> {
        let mut update_doc = updates;
        update_doc.insert("updatedAt", DateTime::now());

        self.collection
            .find_one_and_update(doc! { "momentId": moment_id }, doc! { "$set": update_doc })
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }

    /// Delete a moment by moment_id.
    /// Returns true if a document was deleted.
    pub async fn delete(&self, moment_id: &str) -> Result<bool, String> {
        let result = self
            .collection
            .delete_one(doc! { "momentId": moment_id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.deleted_count > 0)
    }
    /// Update the 0G storage hash for a moment.
    /// Called by the migration worker after successful upload.
    pub async fn update_zg_hash(&self, moment_id: &str, zg_hash: &str) -> Result<bool, String> {
        let result = self
            .collection
            .find_one_and_update(
                doc! { "momentId": moment_id },
                doc! { "$set": { "assetZgHash": zg_hash, "updatedAt": DateTime::now() } },
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.is_some())
    }
}
