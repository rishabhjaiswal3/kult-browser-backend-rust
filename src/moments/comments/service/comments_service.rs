use mongodb::bson::{oid::ObjectId, DateTime};

use crate::handler::AppError;
use crate::moments::comments::dto::{
    CommentListResponse, CommentResponse, CreateCommentRequest, UpdateCommentRequest,
};
use crate::moments::comments::model::CommentModel;
use crate::moments::comments::repository::CommentsRepository;
use crate::moments::repository::MomentsRepository;

const MAX_COMMENT_LENGTH: usize = 500;

#[derive(Clone)]
pub struct CommentsService {
    comments_repository: CommentsRepository,
    moments_repository: MomentsRepository,
}

impl CommentsService {
    pub fn new(
        comments_repository: CommentsRepository,
        moments_repository: MomentsRepository,
    ) -> Self {
        Self {
            comments_repository,
            moments_repository,
        }
    }

    pub async fn create_comment(
        &self,
        moment_id: &str,
        wallet: &str,
        request: CreateCommentRequest,
    ) -> Result<CommentResponse, AppError> {
        self.ensure_moment_exists(moment_id).await?;
        let content = Self::validate_content(&request.content)?;
        let now = DateTime::now();

        let comment = CommentModel {
            id: Some(ObjectId::new()),
            moment_id: moment_id.to_string(),
            parent_comment_id: None,
            author_wallet_address: wallet.trim().to_string(),
            content,
            reply_count: 0,
            is_edited: false,
            is_deleted: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        let created = self
            .comments_repository
            .create(comment)
            .await
            .map_err(AppError::Internal)?;

        let created_id = created
            .id
            .as_ref()
            .ok_or_else(|| AppError::Internal("Comment ID missing after create".to_string()))?
            .to_owned();

        if let Err(e) = self.increment_moment_comment_count(moment_id, 1).await {
            let _ = self.comments_repository.delete(&created_id).await;
            return Err(e);
        }

        Ok(Self::to_response(created))
    }

    pub async fn list_comments(
        &self,
        moment_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<CommentListResponse, AppError> {
        self.ensure_moment_exists(moment_id).await?;
        let (page, per_page) = Self::sanitize_pagination(page, per_page);

        let comments = self
            .comments_repository
            .find_top_level_by_moment(moment_id, page, per_page)
            .await
            .map_err(AppError::Internal)?;
        let total = self
            .comments_repository
            .count_top_level_by_moment(moment_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(CommentListResponse {
            comments: comments.into_iter().map(Self::to_response).collect(),
            total,
            page,
            per_page,
        })
    }

    pub async fn create_reply(
        &self,
        parent_comment_id: &str,
        wallet: &str,
        request: CreateCommentRequest,
    ) -> Result<CommentResponse, AppError> {
        let parent_id = Self::parse_comment_id(parent_comment_id)?;
        let parent = self
            .comments_repository
            .find_by_id(&parent_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if parent.parent_comment_id.is_some() {
            return Err(AppError::BadRequest(
                "Replies to replies are not allowed".to_string(),
            ));
        }

        if parent.is_deleted {
            return Err(AppError::BadRequest(
                "Cannot reply to a deleted comment".to_string(),
            ));
        }

        let content = Self::validate_content(&request.content)?;
        let now = DateTime::now();
        let reply = CommentModel {
            id: Some(ObjectId::new()),
            moment_id: parent.moment_id.clone(),
            parent_comment_id: Some(parent_id),
            author_wallet_address: wallet.trim().to_string(),
            content,
            reply_count: 0,
            is_edited: false,
            is_deleted: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        let created = self
            .comments_repository
            .create(reply)
            .await
            .map_err(AppError::Internal)?;

        let created_id = created
            .id
            .as_ref()
            .ok_or_else(|| AppError::Internal("Comment ID missing after create".to_string()))?
            .to_owned();

        if let Err(e) = self
            .comments_repository
            .increment_reply_count(&parent_id, 1)
            .await
        {
            let _ = self.comments_repository.delete(&created_id).await;
            return Err(AppError::Internal(e));
        }

        if let Err(e) = self
            .increment_moment_comment_count(&parent.moment_id, 1)
            .await
        {
            let _ = self
                .comments_repository
                .increment_reply_count(&parent_id, -1)
                .await;
            let _ = self.comments_repository.delete(&created_id).await;
            return Err(e);
        }

        Ok(Self::to_response(created))
    }

    pub async fn list_replies(
        &self,
        parent_comment_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<CommentListResponse, AppError> {
        let parent_id = Self::parse_comment_id(parent_comment_id)?;
        let parent = self
            .comments_repository
            .find_by_id(&parent_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if parent.parent_comment_id.is_some() {
            return Err(AppError::BadRequest(
                "Replies can only be listed for top-level comments".to_string(),
            ));
        }

        let (page, per_page) = Self::sanitize_pagination(page, per_page);
        let comments = self
            .comments_repository
            .find_replies(&parent_id, page, per_page)
            .await
            .map_err(AppError::Internal)?;
        let total = self
            .comments_repository
            .count_replies(&parent_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(CommentListResponse {
            comments: comments.into_iter().map(Self::to_response).collect(),
            total,
            page,
            per_page,
        })
    }

    pub async fn update_comment(
        &self,
        wallet: &str,
        comment_id: &str,
        request: UpdateCommentRequest,
    ) -> Result<CommentResponse, AppError> {
        let comment_id = Self::parse_comment_id(comment_id)?;
        let existing = self
            .comments_repository
            .find_by_id(&comment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if existing.author_wallet_address != wallet.trim() {
            return Err(AppError::Forbidden(
                "You can only update your own comments".to_string(),
            ));
        }

        if existing.is_deleted {
            return Err(AppError::BadRequest(
                "Cannot edit a deleted comment".to_string(),
            ));
        }

        let content = Self::validate_content(&request.content)?;
        let updated = self
            .comments_repository
            .update_content(&comment_id, &content)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        Ok(Self::to_response(updated))
    }

    pub async fn delete_comment(&self, wallet: &str, comment_id: &str) -> Result<(), AppError> {
        let comment_id = Self::parse_comment_id(comment_id)?;
        let existing = self
            .comments_repository
            .find_by_id(&comment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if existing.author_wallet_address != wallet.trim() {
            return Err(AppError::Forbidden(
                "You can only delete your own comments".to_string(),
            ));
        }

        if existing.is_deleted {
            return Err(AppError::BadRequest("Comment already deleted".to_string()));
        }

        if let Some(parent_comment_id) = existing.parent_comment_id {
            self.comments_repository
                .delete(&comment_id)
                .await
                .map_err(AppError::Internal)?;

            self.comments_repository
                .increment_reply_count(&parent_comment_id, -1)
                .await
                .map_err(AppError::Internal)?;

            self.increment_moment_comment_count(&existing.moment_id, -1)
                .await?;

            if let Some(parent) = self
                .comments_repository
                .find_by_id(&parent_comment_id)
                .await
                .map_err(AppError::Internal)?
            {
                if parent.is_deleted && parent.reply_count == 0 {
                    let _ = self.comments_repository.delete(&parent_comment_id).await;
                }
            }

            return Ok(());
        }

        if existing.reply_count > 0 {
            self.comments_repository
                .soft_delete(&comment_id)
                .await
                .map_err(AppError::Internal)?;
            self.increment_moment_comment_count(&existing.moment_id, -1)
                .await?;
        } else {
            self.comments_repository
                .delete(&comment_id)
                .await
                .map_err(AppError::Internal)?;
            self.increment_moment_comment_count(&existing.moment_id, -1)
                .await?;
        }

        Ok(())
    }

    async fn ensure_moment_exists(&self, moment_id: &str) -> Result<(), AppError> {
        self.moments_repository
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;
        Ok(())
    }

    fn parse_comment_id(comment_id: &str) -> Result<ObjectId, AppError> {
        ObjectId::parse_str(comment_id)
            .map_err(|_| AppError::BadRequest("Invalid comment_id".to_string()))
    }

    fn validate_content(content: &str) -> Result<String, AppError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::BadRequest("content is required".to_string()));
        }
        if content.len() > MAX_COMMENT_LENGTH {
            return Err(AppError::BadRequest(format!(
                "content cannot exceed {} characters",
                MAX_COMMENT_LENGTH
            )));
        }
        Ok(content.to_string())
    }

    fn sanitize_pagination(page: u32, per_page: u32) -> (u32, u32) {
        let page = if page == 0 { 1 } else { page };
        let per_page = per_page.min(50).max(1);
        (page, per_page)
    }

    async fn increment_moment_comment_count(
        &self,
        moment_id: &str,
        delta: i64,
    ) -> Result<(), AppError> {
        let updated = self
            .moments_repository
            .increment_num_comments(moment_id, delta)
            .await
            .map_err(AppError::Internal)?;

        if !updated {
            return Err(AppError::NotFound("Moment not found".to_string()));
        }

        Ok(())
    }

    fn to_response(comment: CommentModel) -> CommentResponse {
        CommentResponse {
            comment_id: comment.id.map(|id| id.to_hex()).unwrap_or_default(),
            moment_id: comment.moment_id,
            parent_comment_id: comment.parent_comment_id.map(|id| id.to_hex()),
            author_wallet_address: comment.author_wallet_address,
            content: if comment.is_deleted {
                None
            } else {
                Some(comment.content)
            },
            reply_count: comment.reply_count,
            is_edited: comment.is_edited,
            is_deleted: comment.is_deleted,
            created_at: comment
                .created_at
                .try_to_rfc3339_string()
                .unwrap_or_default(),
            updated_at: comment
                .updated_at
                .try_to_rfc3339_string()
                .unwrap_or_default(),
            deleted_at: comment
                .deleted_at
                .and_then(|dt| dt.try_to_rfc3339_string().ok()),
        }
    }
}
