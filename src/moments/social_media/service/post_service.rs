use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use reqwest::Url;
use std::fmt;

use super::super::{
    dto::{SharedPostListResponse, SharedPostResponse},
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
    /// Platform is not supported by the worker/scraper
    UnsupportedPlatform,
    /// URL is malformed or doesn't match the selected platform
    InvalidPlatformUrl(String),
    /// Referenced moment does not exist
    MomentNotFound,
    /// Authenticated wallet does not own the referenced moment
    ForbiddenMomentAccess,
    /// Shared post does not exist
    PostNotFound,
    /// Queue is unavailable so validation cannot be scheduled
    QueueUnavailable,
    /// Queue push failed after the post was created
    QueuePushFailed(String),
    /// A database or infrastructure error
    Database(String),
}

impl fmt::Display for PostServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePost => write!(f, "This post has already been submitted"),
            Self::UnsupportedPlatform => write!(f, "This platform is not supported yet"),
            Self::InvalidPlatformUrl(msg) => write!(f, "{msg}"),
            Self::MomentNotFound => write!(f, "Moment not found"),
            Self::ForbiddenMomentAccess => {
                write!(f, "You can only submit posts for your own moments")
            }
            Self::PostNotFound => write!(f, "Shared post not found"),
            Self::QueueUnavailable => {
                write!(f, "Validation queue unavailable. Please try again later")
            }
            Self::QueuePushFailed(msg) => write!(f, "Failed to schedule validation: {msg}"),
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
        let wallet_address = wallet_address.trim().to_string();
        let post_id = post_id.trim().to_string();
        let url = url.trim().to_string();

        Self::ensure_supported_platform(&platform)?;
        Self::validate_platform_url(&platform, &url)?;

        let queue = self
            .queue
            .as_ref()
            .ok_or(PostServiceError::QueueUnavailable)?;

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
        let scrape_job = ScrapeJob {
            post_db_id: inserted_id,
            platform,
            url,
            created_at: now,
            attempt: 1,
        };

        if let Err(e) = queue.push_async(&scrape_job).await {
            tracing::error!(
                post_db_id = %inserted_id,
                error = %e,
                "Failed to queue scrape job — rolling back shared post"
            );
            let _ = self.post_repository.delete_post(inserted_id).await;
            return Err(PostServiceError::QueuePushFailed(e));
        }

        tracing::info!(
            post_db_id = %inserted_id,
            "Scrape job queued for delayed validation"
        );

        Ok(inserted_id)
    }

    pub async fn list_shared_posts(
        &self,
        wallet_address: String,
    ) -> Result<SharedPostListResponse, PostServiceError> {
        let wallet_address = wallet_address.trim().to_string();
        let posts = self
            .post_repository
            .get_posts_by_wallet_address(&wallet_address)
            .await?;

        Ok(SharedPostListResponse {
            posts: posts.into_iter().map(Self::to_response).collect(),
        })
    }

    pub async fn requeue_shared_post(
        &self,
        post_id: String,
        wallet_address: String,
    ) -> Result<ObjectId, PostServiceError> {
        let queue = self
            .queue
            .as_ref()
            .ok_or(PostServiceError::QueueUnavailable)?;
        let post_object_id = ObjectId::parse_str(post_id.trim()).map_err(|_| {
            PostServiceError::InvalidPlatformUrl("Invalid shared post id".to_string())
        })?;
        let wallet_address = wallet_address.trim().to_string();

        let post = self
            .post_repository
            .get_post_by_id(post_object_id)
            .await?
            .ok_or(PostServiceError::PostNotFound)?;

        if post.wallet_address != wallet_address {
            return Err(PostServiceError::ForbiddenMomentAccess);
        }

        Self::ensure_supported_platform(&post.platform)?;
        Self::validate_platform_url(&post.platform, &post.url)?;

        let scrape_job = ScrapeJob {
            post_db_id: post_object_id,
            platform: post.platform.clone(),
            url: post.url.clone(),
            created_at: post.created_at,
            attempt: 1,
        };

        queue
            .push_async(&scrape_job)
            .await
            .map_err(PostServiceError::QueuePushFailed)?;

        if let Err(e) = self.post_repository.mark_post_pending(post_object_id).await {
            tracing::warn!(
                post_db_id = %post_object_id,
                error = %e,
                "Failed to mark post pending before requeue"
            );
        }

        Ok(post_object_id)
    }

    fn ensure_supported_platform(platform: &Platform) -> Result<(), PostServiceError> {
        match platform {
            Platform::Farcaster => Err(PostServiceError::UnsupportedPlatform),
            _ => Ok(()),
        }
    }

    fn validate_platform_url(platform: &Platform, url: &str) -> Result<(), PostServiceError> {
        let parsed = Url::parse(url).map_err(|_| {
            PostServiceError::InvalidPlatformUrl("Social media URL must be a valid URL".to_string())
        })?;
        let host = parsed.host_str().ok_or_else(|| {
            PostServiceError::InvalidPlatformUrl(
                "Social media URL must include a valid hostname".to_string(),
            )
        })?;

        let host = host.to_lowercase();
        let allowed_domains = match platform {
            Platform::Twitter => &["x.com", "twitter.com"][..],
            Platform::Instagram => &["instagram.com"][..],
            Platform::TikTok => &["tiktok.com"][..],
            Platform::Facebook => &["facebook.com", "fb.watch"][..],
            Platform::Reddit => &["reddit.com", "redd.it"][..],
            Platform::LinkedIn => &["linkedin.com"][..],
            Platform::Pinterest => &["pinterest.com", "pin.it"][..],
            Platform::Farcaster => &[][..],
        };

        let matches_domain = allowed_domains
            .iter()
            .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")));

        if !matches_domain {
            return Err(PostServiceError::InvalidPlatformUrl(format!(
                "URL host does not match selected platform {:?}",
                platform
            )));
        }

        Ok(())
    }

    fn to_response(post: SharedPost) -> SharedPostResponse {
        SharedPostResponse {
            id: post.id.map(|id| id.to_hex()).unwrap_or_default(),
            moment_id: Self::moment_id_to_string(post.moment_id),
            platform: post.platform,
            external_post_id: post.post_id,
            url: post.url,
            num_likes: post.num_likes,
            score: post.score,
            is_validated: post.is_validated,
            validation_status: post.validation_status,
            validation_reason: post.validation_reason,
            last_validated_at: post.last_validated_at.map(|dt| dt.to_rfc3339()),
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }

    fn moment_id_to_string(moment_id: Bson) -> String {
        match moment_id {
            Bson::String(value) => value,
            Bson::ObjectId(oid) => oid.to_hex(),
            other => other.to_string(),
        }
    }
}
