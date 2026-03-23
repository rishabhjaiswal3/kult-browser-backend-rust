use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use std::fmt;

use super::super::{
    model::{
        platform::Platform,
        post_model::{SharedPost, ValidationStatus},
    },
    repository::post_repository::PostRepository,
    worker::scrape_job::ScrapeJob,
};
use crate::moments::repository::MomentsRepository;
use crate::redis::ValkyQueue;

/// Errors that can occur when submitting a shared post
#[derive(Debug)]
pub enum PostServiceError {
    /// The same post_id + platform combination was already submitted
    DuplicatePost,
    /// Referenced moment does not exist
    MomentNotFound,
    /// Authenticated wallet does not own the referenced moment
    ForbiddenMomentAccess,
    /// A database or infrastructure error
    Database(String),
}

impl fmt::Display for PostServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePost => write!(f, "This post has already been submitted"),
            Self::MomentNotFound => write!(f, "Moment not found"),
            Self::ForbiddenMomentAccess => {
                write!(f, "You can only submit posts for your own moments")
            }
            Self::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl From<mongodb::error::Error> for PostServiceError {
    fn from(e: mongodb::error::Error) -> Self {
        Self::Database(e.to_string())
    }
}

#[derive(Clone)]
pub struct PostService {
    post_repository: PostRepository,
    moments_repository: MomentsRepository,
    queue: Option<ValkyQueue>,
}

impl PostService {
    /// Create without queue (for tests or when queue is not available).
    pub fn new(post_repository: PostRepository, moments_repository: MomentsRepository) -> Self {
        Self {
            post_repository,
            moments_repository,
            queue: None,
        }
    }

    /// Create with queue for production use — scrape jobs will be pushed automatically.
    pub fn with_queue(
        post_repository: PostRepository,
        moments_repository: MomentsRepository,
        queue: ValkyQueue,
    ) -> Self {
        Self {
            post_repository,
            moments_repository,
            queue: Some(queue),
        }
    }

    /// Submits a post representing a player sharing a moment.
    /// It defaults to 0 likes, 0 score, and 'Pending' validation.
    /// If a queue is configured, a ScrapeJob is automatically pushed for delayed validation.
    pub async fn submit_shared_post(
        &self,
        moment_id: String,
        wallet_address: String,
        platform: Platform,
        post_id: String,
        url: String,
    ) -> Result<ObjectId, PostServiceError> {
        let moment_id = moment_id.trim().to_string();
        let wallet_address = wallet_address.trim().to_lowercase();
        let post_id = post_id.trim().to_string();
        let url = url.trim().to_string();

        let moment = self
            .moments_repository
            .find_by_moment_id(&moment_id)
            .await
            .map_err(PostServiceError::Database)?
            .ok_or(PostServiceError::MomentNotFound)?;

        if moment.player_wallet_address != wallet_address {
            return Err(PostServiceError::ForbiddenMomentAccess);
        }

        // First check if this post id was already submitted for this platform
        if let Some(_existing) = self
            .post_repository
            .get_post_by_platform_and_id(platform.clone(), &post_id)
            .await?
        {
            return Err(PostServiceError::DuplicatePost);
        }

        let now = Utc::now();

        let new_post = SharedPost {
            id: None,
            moment_id: Bson::String(moment_id),
            wallet_address,
            platform: platform.clone(),
            post_id,
            url: url.clone(),
            num_likes: 0,
            score: 0,
            is_validated: false,
            validation_status: ValidationStatus::Pending,
            validation_reason: None,
            last_validated_at: None,
            created_at: now,
            updated_at: now,
        };

        let inserted_id = self.post_repository.create_post(new_post).await?;

        // Push scrape job to queue for delayed validation
        if let Some(ref queue) = self.queue {
            let scrape_job = ScrapeJob {
                post_db_id: inserted_id,
                platform,
                url,
                created_at: now,
                attempt: 1,
            };

            match queue.push_async(&scrape_job).await {
                Ok(_) => {
                    tracing::info!(
                        post_db_id = %inserted_id,
                        "Scrape job queued for delayed validation"
                    );
                }
                Err(e) => {
                    // Don't fail the submission if queue push fails —
                    // the post is already saved, and we can manually re-queue later
                    tracing::error!(
                        post_db_id = %inserted_id,
                        error = %e,
                        "Failed to queue scrape job — post saved but validation not scheduled"
                    );
                }
            }
        } else {
            tracing::warn!(
                post_db_id = %inserted_id,
                "No queue configured — scrape job not queued"
            );
        }

        Ok(inserted_id)
    }
}
