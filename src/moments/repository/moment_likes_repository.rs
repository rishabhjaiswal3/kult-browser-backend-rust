use mongodb::{bson::doc, Collection, Database};

use crate::config::CONFIG;
use crate::moments::model::MomentLikeModel;

#[derive(Clone)]
pub struct MomentLikesRepository {
    collection: Collection<MomentLikeModel>,
}

pub enum CreateMomentLikeError {
    Duplicate,
    Internal(String),
}

impl MomentLikesRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.moment_likes_collection),
        }
    }

    pub async fn create(&self, like: MomentLikeModel) -> Result<(), CreateMomentLikeError> {
        self.collection
            .insert_one(like)
            .await
            .map(|_| ())
            .map_err(|e| {
                if e.to_string().contains("E11000") {
                    CreateMomentLikeError::Duplicate
                } else {
                    CreateMomentLikeError::Internal(e.to_string())
                }
            })
    }

    pub async fn delete_by_id(&self, like_id: &str) -> Result<bool, String> {
        let result = self
            .collection
            .delete_one(doc! { "_id": like_id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.deleted_count > 0)
    }
}
