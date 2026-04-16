// src/marketplace/repository/listing_repository.rs

use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use mongodb::{Collection, Database};

use crate::config::CONFIG;
use crate::marketplace::model::ListingModel;

#[derive(Clone)]
pub struct ListingRepository {
    collection: Collection<ListingModel>,
    raw_collection: Collection<Document>,
}

impl ListingRepository {
    pub fn new(db: &Database) -> Self {
        let name = &CONFIG.db.marketplace_listings_collection;
        Self {
            collection: db.collection(name),
            raw_collection: db.collection(name),
        }
    }

    /// Create a new listing.
    pub async fn create(&self, listing: ListingModel) -> Result<ObjectId, String> {
        let mut doc = mongodb::bson::to_document(&listing).map_err(|e| e.to_string())?;
        let now = DateTime::now();
        doc.insert("createdAt", now);
        doc.insert("updatedAt", now);

        let result = self
            .raw_collection
            .insert_one(doc)
            .await
            .map_err(|e| e.to_string())?;

        result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| "Failed to get inserted ID".to_string())
    }

    /// Find a listing by ObjectId.
    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<ListingModel>, String> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| e.to_string())
    }

    /// Find all active listings (paginated), optionally filtered by game identification and category.
    pub async fn find_active(
        &self,
        game_identification: Option<&str>,
        category: Option<&str>,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<ListingModel>, String> {
        let mut filter = doc! { "status": "active" };

        if let Some(gi) = game_identification {
            filter.insert("gameIdentification", gi);
        }
        if let Some(cat) = category {
            filter.insert("category", cat);
        }

        let cursor = self
            .collection
            .find(filter)
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Count active listings with optional filters.
    pub async fn count_active(
        &self,
        game_identification: Option<&str>,
        category: Option<&str>,
    ) -> Result<u64, String> {
        let mut filter = doc! { "status": "active" };

        if let Some(gi) = game_identification {
            filter.insert("gameIdentification", gi);
        }
        if let Some(cat) = category {
            filter.insert("category", cat);
        }

        self.collection
            .count_documents(filter)
            .await
            .map_err(|e| e.to_string())
    }

    /// Update a listing by ID.
    pub async fn update(
        &self,
        id: &ObjectId,
        updates: Document,
    ) -> Result<Option<ListingModel>, String> {
        let mut set_doc = updates;
        set_doc.insert("updatedAt", DateTime::now());

        self.collection
            .find_one_and_update(doc! { "_id": id }, doc! { "$set": set_doc })
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }
}
