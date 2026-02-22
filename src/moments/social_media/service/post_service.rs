use chrono::Utc;
use mongodb::bson::oid::ObjectId;

use super::super::{
    model::{
        platform::Platform,
        post_model::{SharedPost, ValidationStatus},
    },
    repository::post_repository::PostRepository,
};

pub struct PostService {
    repository: PostRepository,
}

impl PostService {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        let repository = PostRepository::new().await?;
        Ok(Self { repository })
    }

    /// Submits a post representing a player sharing a moment.
    /// It defaults to 0 likes, 0 score, and 'Pending' validation.
    pub async fn submit_shared_post(
        &self,
        moment_id: ObjectId,
        wallet_address: String,
        platform: Platform,
        post_id: String,
        url: String,
    ) -> Result<ObjectId, mongodb::error::Error> {
        // First check if this post id was already submitted for this platform
        if let Some(_existing) = self
            .repository
            .get_post_by_platform_and_id(platform.clone(), &post_id)
            .await?
        {
            // Ideally return a custom error like DuplicatePostError, but returning a generic Mongo error for now
            // as placeholder for the struct implementation
            return Err(mongodb::error::Error::custom("Post already exists"));
        }

        let now = Utc::now();

        let new_post = SharedPost {
            id: None, // Mongo will generate this
            moment_id,
            wallet_address,
            platform,
            post_id,
            url,
            num_likes: 0,
            score: 0,
            is_validated: false,
            validation_status: ValidationStatus::Pending,
            last_validated_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repository.create_post(new_post).await
    }
}
