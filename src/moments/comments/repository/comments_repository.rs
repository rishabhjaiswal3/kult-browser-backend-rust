use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Bson, DateTime};
use mongodb::{options::ReturnDocument, Collection, Database};

use crate::config::CONFIG;
use crate::moments::comments::model::CommentModel;

#[derive(Clone)]
pub struct CommentsRepository {
    collection: Collection<CommentModel>,
}

impl CommentsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.moment_comments_collection),
        }
    }

    pub async fn create(&self, comment: CommentModel) -> Result<CommentModel, String> {
        self.collection
            .insert_one(comment.clone())
            .await
            .map_err(|e| e.to_string())?;
        Ok(comment)
    }

    pub async fn find_by_id(&self, comment_id: &ObjectId) -> Result<Option<CommentModel>, String> {
        self.collection
            .find_one(doc! { "_id": comment_id })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn find_top_level_by_moment(
        &self,
        moment_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<CommentModel>, String> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let cursor = self
            .collection
            .find(doc! {
                "momentId": moment_id,
                "parentCommentId": Bson::Null
            })
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(per_page as i64)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    pub async fn count_top_level_by_moment(&self, moment_id: &str) -> Result<u64, String> {
        self.collection
            .count_documents(doc! {
                "momentId": moment_id,
                "parentCommentId": Bson::Null
            })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn find_replies(
        &self,
        parent_comment_id: &ObjectId,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<CommentModel>, String> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let cursor = self
            .collection
            .find(doc! { "parentCommentId": parent_comment_id })
            .sort(doc! { "createdAt": 1 })
            .skip(skip)
            .limit(per_page as i64)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    pub async fn count_replies(&self, parent_comment_id: &ObjectId) -> Result<u64, String> {
        self.collection
            .count_documents(doc! { "parentCommentId": parent_comment_id })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_content(
        &self,
        comment_id: &ObjectId,
        content: &str,
    ) -> Result<Option<CommentModel>, String> {
        self.collection
            .find_one_and_update(
                doc! { "_id": comment_id },
                doc! {
                    "$set": {
                        "content": content,
                        "isEdited": true,
                        "updatedAt": DateTime::now()
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn soft_delete(&self, comment_id: &ObjectId) -> Result<Option<CommentModel>, String> {
        let now = DateTime::now();
        self.collection
            .find_one_and_update(
                doc! { "_id": comment_id },
                doc! {
                    "$set": {
                        "content": "",
                        "isDeleted": true,
                        "deletedAt": now,
                        "updatedAt": now
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete(&self, comment_id: &ObjectId) -> Result<bool, String> {
        let result = self
            .collection
            .delete_one(doc! { "_id": comment_id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.deleted_count > 0)
    }

    pub async fn increment_reply_count(
        &self,
        comment_id: &ObjectId,
        delta: i32,
    ) -> Result<bool, String> {
        let result = self
            .collection
            .update_one(
                doc! { "_id": comment_id },
                doc! {
                    "$inc": { "replyCount": delta },
                    "$set": { "updatedAt": DateTime::now() }
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.matched_count > 0)
    }
}
