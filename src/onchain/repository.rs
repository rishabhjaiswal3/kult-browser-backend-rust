use futures::TryStreamExt;
use mongodb::bson::{doc, DateTime};
use mongodb::{Collection, Database};

use crate::config::CONFIG;
use crate::onchain::model::OnchainActivityJob;

#[derive(Clone)]
pub struct OnchainActivityRepository {
    collection: Collection<OnchainActivityJob>,
}

impl OnchainActivityRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.onchain_activity_jobs_collection),
        }
    }

    pub async fn create_pending(&self, job: OnchainActivityJob) -> Result<(), String> {
        self.collection
            .insert_one(job)
            .await
            .map(|_| ())
            .map_err(|e| {
                if e.to_string().contains("E11000") {
                    "duplicate activity job".to_string()
                } else {
                    e.to_string()
                }
            })
    }

    pub async fn find_pending(&self, limit: i64) -> Result<Vec<OnchainActivityJob>, String> {
        let cursor = self
            .collection
            .find(doc! { "status": "pending" })
            .sort(doc! { "createdAt": 1 })
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    pub async fn mark_submitted(&self, activity_id: &str, tx_hash: &str) -> Result<(), String> {
        self.collection
            .update_one(
                doc! { "activityId": activity_id },
                doc! {
                    "$set": {
                        "status": "submitted",
                        "txHash": tx_hash,
                        "updatedAt": DateTime::now()
                    },
                    "$inc": { "attempts": 1 }
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn mark_confirmed(&self, activity_id: &str) -> Result<(), String> {
        self.collection
            .update_one(
                doc! { "activityId": activity_id },
                doc! {
                    "$set": {
                        "status": "confirmed",
                        "updatedAt": DateTime::now()
                    }
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn mark_failed(&self, activity_id: &str, error: &str) -> Result<(), String> {
        self.collection
            .update_one(
                doc! { "activityId": activity_id },
                doc! {
                    "$set": {
                        "status": "failed",
                        "lastError": error,
                        "updatedAt": DateTime::now()
                    },
                    "$inc": { "attempts": 1 }
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn reset_for_retry(&self, activity_id: &str, error: &str) -> Result<(), String> {
        self.collection
            .update_one(
                doc! { "activityId": activity_id },
                doc! {
                    "$set": {
                        "status": "pending",
                        "lastError": error,
                        "updatedAt": DateTime::now()
                    },
                    "$inc": { "attempts": 1 }
                },
            )
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
