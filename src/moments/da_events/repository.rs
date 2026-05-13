// src/moments/da_events/repository.rs

use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use mongodb::{Collection, Database};

use crate::external::da::DaReceipt;
use crate::moments::da_events::model::{MomentDAEventModel, DA_STATUS_DISPERSING, DA_STATUS_FAILED, DA_STATUS_FINALIZED, DA_STATUS_PENDING};

#[derive(Clone)]
pub struct MomentDAEventRepository {
    collection: Collection<MomentDAEventModel>,
    raw_collection: Collection<Document>,
}

impl MomentDAEventRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("momentDaEvents"),
            raw_collection: db.collection("momentDaEvents"),
        }
    }

    pub async fn create(&self, event: MomentDAEventModel) -> Result<(), String> {
        let mut doc = mongodb::bson::to_document(&event).map_err(|e| e.to_string())?;
        let now = DateTime::now();
        doc.insert("createdAt", now);
        doc.insert("updatedAt", now);
        self.raw_collection
            .insert_one(doc)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Find events not yet submitted to the DA disperser.
    pub async fn find_pending(&self, limit: i64) -> Result<Vec<MomentDAEventModel>, String> {
        let cursor = self
            .collection
            .find(doc! { "daStatus": DA_STATUS_PENDING })
            .sort(doc! { "createdAt": 1 })
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;
        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Find events submitted to the disperser but not yet finalized.
    pub async fn find_dispersing(&self, limit: i64) -> Result<Vec<MomentDAEventModel>, String> {
        let cursor = self
            .collection
            .find(doc! { "daStatus": DA_STATUS_DISPERSING })
            .sort(doc! { "createdAt": 1 })
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;
        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    pub async fn find_by_moment_id(
        &self,
        moment_id: &str,
    ) -> Result<Vec<MomentDAEventModel>, String> {
        let cursor = self
            .collection
            .find(doc! { "momentId": moment_id })
            .sort(doc! { "createdAt": 1 })
            .await
            .map_err(|e| e.to_string())?;
        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Mark a blob as submitted to the disperser — stores the requestId.
    pub async fn mark_dispersing(&self, id: &ObjectId, request_id: &str) -> Result<(), String> {
        self.raw_collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": {
                    "daStatus": DA_STATUS_DISPERSING,
                    "daRequestId": request_id,
                    "daError": mongodb::bson::Bson::Null,
                    "updatedAt": DateTime::now(),
                }},
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Mark a blob as DA-finalized — stores the full receipt from the disperser.
    pub async fn mark_finalized(&self, id: &ObjectId, receipt: &DaReceipt) -> Result<(), String> {
        let mut set_doc = doc! {
            "daStatus": DA_STATUS_FINALIZED,
            "daError": mongodb::bson::Bson::Null,
            "updatedAt": DateTime::now(),
        };

        if let Some(batch_id) = receipt.batch_id {
            set_doc.insert("daBatchId", batch_id as i64);
        }
        if let Some(blob_index) = receipt.blob_index {
            set_doc.insert("daBlobIndex", blob_index as i64);
        }
        if let Some(ref hash) = receipt.batch_header_hash {
            set_doc.insert("daBatchHeaderHash", hash.as_str());
        }
        if let Some(block) = receipt.confirmation_block {
            set_doc.insert("daConfirmationBlock", block as i64);
        }
        if let Some(ref ts) = receipt.finalized_at {
            set_doc.insert("daFinalizedAt", ts.as_str());
        }

        self.raw_collection
            .update_one(doc! { "_id": id }, doc! { "$set": set_doc })
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: &ObjectId, error: &str) -> Result<(), String> {
        self.raw_collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": {
                    "daStatus": DA_STATUS_FAILED,
                    "daError": error,
                    "updatedAt": DateTime::now(),
                }},
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
