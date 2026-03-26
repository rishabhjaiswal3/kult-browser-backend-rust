// src/moments/service/moments_service.rs

use crate::external::digital_ocean::spaces::SpacesService;
use crate::handler::AppError;
use crate::moments::dto::{
    CreateMomentRequest, CreateMomentResponse, LikeMomentResponse, MomentListResponse,
    MomentResponse,
    UpdateMomentRequest,
};
use crate::moments::model::{MomentLikeModel, MomentModel};
use crate::moments::repository::{CreateMomentLikeError, MomentLikesRepository, MomentsRepository};
use crate::moments::worker::migration_worker::MigrationJob;
use crate::redis::ValkyQueue;

/// Service layer for Moment operations.
#[derive(Clone)]
pub struct MomentsService {
    repo: MomentsRepository,
    likes_repo: MomentLikesRepository,
    queue: Option<ValkyQueue>,
    spaces_service: SpacesService,
}

impl MomentsService {
    pub fn new(
        repo: MomentsRepository,
        likes_repo: MomentLikesRepository,
        spaces_service: SpacesService,
    ) -> Self {
        Self {
            repo,
            likes_repo,
            queue: None,
            spaces_service,
        }
    }

    /// Create with queue for migration support.
    pub fn with_queue(
        repo: MomentsRepository,
        likes_repo: MomentLikesRepository,
        queue: ValkyQueue,
        spaces_service: SpacesService,
    ) -> Self {
        Self {
            repo,
            likes_repo,
            queue: Some(queue),
            spaces_service,
        }
    }

    /// Create a new moment.
    pub async fn create_moment(
        &self,
        wallet: &str,
        request: CreateMomentRequest,
    ) -> Result<CreateMomentResponse, AppError> {
        tracing::debug!(wallet = %wallet, title = %request.title, "Creating new moment");

        // Validate required fields
        if request.title.trim().is_empty() {
            return Err(AppError::BadRequest("title is required".to_string()));
        }
        if request.title.len() > 200 {
            return Err(AppError::BadRequest(
                "title cannot exceed 200 characters".to_string(),
            ));
        }
        if let Some(ref desc) = request.description {
            if desc.len() > 2000 {
                return Err(AppError::BadRequest(
                    "description cannot exceed 2000 characters".to_string(),
                ));
            }
        }
        if request.tags.len() > 10 {
            return Err(AppError::BadRequest(
                "cannot have more than 10 tags".to_string(),
            ));
        }
        if let Some(asset_url) = request.asset_url.as_deref().map(str::trim) {
            if asset_url.is_empty() {
                return Err(AppError::BadRequest(
                    "asset_url cannot be empty".to_string(),
                ));
            }
            if !self.spaces_service.check_file_exists(asset_url).await {
                tracing::warn!(url = %asset_url, "Asset URL provided but file not found in storage");
                return Err(AppError::BadRequest(
                    "Verify failed: File not found in storage".to_string(),
                ));
            }
        }

        let moment_id = MomentModel::generate_moment_id();

        let moment = MomentModel {
            id: None,
            moment_id: moment_id.clone(),
            player_wallet_address: wallet.trim().to_lowercase(),
            asset_url: request
                .asset_url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(|u| u.to_string()),
            asset_zg_hash: None,
            num_likes: 0,
            num_comments: 0,
            asset_metadata: request
                .asset_metadata
                .as_ref()
                .and_then(|v| mongodb::bson::to_document(v).ok()),
            title: request.title.trim().to_string(),
            description: request.description.as_deref().map(|d| d.trim().to_string()),
            tags: request
                .tags
                .into_iter()
                .map(|t| t.trim().to_string())
                .collect(),
            social_media_links: request
                .social_media_links
                .and_then(|v| mongodb::bson::to_document(&v).ok()),
            created_at: None,
            updated_at: None,
            original_filename: None,
            file_size_bytes: None,
        };

        self.repo.create(moment).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to create moment");
            AppError::Internal(e)
        })?;

        // Push migration job if asset_url and fileType are present
        if let Some(asset_url) = request
            .asset_url
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())
        {
            // Extract fileType from assetMetadata
            let asset_type = request
                .asset_metadata
                .as_ref()
                .and_then(|m| m.get("fileType"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(asset_type) = asset_type {
                let job = MigrationJob {
                    asset_url: asset_url.to_string(),
                    asset_id: moment_id.clone(),
                    asset_type,
                    attempt: 1,
                };

                if let Some(ref queue) = self.queue {
                    match queue.push_async(&job).await {
                        Ok(_) => {
                            tracing::info!(
                                moment_id = %moment_id,
                                "Migration job queued"
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                moment_id = %moment_id,
                                "Failed to queue migration job"
                            );
                        }
                    }
                } else {
                    tracing::warn!(
                        moment_id = %moment_id,
                        "No queue configured — migration job not queued"
                    );
                }
            }
        }

        tracing::info!(moment_id = %moment_id, wallet = %wallet, "Moment created successfully");

        Ok(CreateMomentResponse {
            moment_id,
            message: "Moment created successfully".to_string(),
        })
    }

    /// Get a moment by its shareable ID.
    pub async fn get_moment(&self, moment_id: &str) -> Result<MomentResponse, AppError> {
        tracing::debug!(moment_id = %moment_id, "Fetching moment");

        let moment = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch moment");
                AppError::Internal(e)
            })?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        Ok(Self::to_response(moment))
    }

    /// Get public feed of moments.
    pub async fn get_feed(
        &self,
        page: u32,
        per_page: u32,
        tags: Option<Vec<String>>,
    ) -> Result<MomentListResponse, AppError> {
        let page = if page == 0 { 1 } else { page };
        let per_page = per_page.min(50).max(1);

        tracing::debug!(page = page, per_page = per_page, "Fetching moments feed");

        let moments = self
            .repo
            .find_all(page, per_page, tags.clone())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch feed");
                AppError::Internal(e)
            })?;

        let total = self.repo.count_all(tags).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to count moments");
            AppError::Internal(e)
        })?;

        Ok(MomentListResponse {
            moments: moments.into_iter().map(Self::to_response).collect(),
            total,
            page,
            per_page,
        })
    }

    /// Get moments for a specific player.
    pub async fn get_player_moments(
        &self,
        wallet: &str,
        page: u32,
        per_page: u32,
    ) -> Result<MomentListResponse, AppError> {
        let wallet = wallet.trim().to_lowercase();
        let page = if page == 0 { 1 } else { page };
        let per_page = per_page.min(50).max(1);

        tracing::debug!(wallet = %wallet, page = page, "Fetching player moments");

        let moments = self
            .repo
            .find_by_wallet(&wallet, page, per_page)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch player moments");
                AppError::Internal(e)
            })?;

        let total = self.repo.count_by_wallet(&wallet).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to count player moments");
            AppError::Internal(e)
        })?;

        Ok(MomentListResponse {
            moments: moments.into_iter().map(Self::to_response).collect(),
            total,
            page,
            per_page,
        })
    }

    /// Update a moment (only if owned by the wallet).
    pub async fn update_moment(
        &self,
        wallet: &str,
        moment_id: &str,
        request: UpdateMomentRequest,
    ) -> Result<MomentResponse, AppError> {
        tracing::debug!(wallet = %wallet, moment_id = %moment_id, "Updating moment");

        // First verify ownership
        let existing = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        if existing.player_wallet_address != wallet.trim().to_lowercase() {
            tracing::warn!(wallet = %wallet, owner = %existing.player_wallet_address, "Unauthorized update attempt");
            return Err(AppError::Forbidden(
                "You can only update your own moments".to_string(),
            ));
        }

        // Build update document
        let mut updates = mongodb::bson::Document::new();

        if let Some(url) = request.asset_url {
            if url.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "asset_url cannot be empty".to_string(),
                ));
            }

            // Verify file exists in storage
            if !self.spaces_service.check_file_exists(url.trim()).await {
                tracing::warn!(url = %url, "Asset URL provided but file not found in storage");
                return Err(AppError::BadRequest(
                    "Verify failed: File not found in storage".to_string(),
                ));
            }

            updates.insert("assetUrl", url.trim());
        }
        if let Some(filename) = request.original_filename {
            if !filename.trim().is_empty() {
                updates.insert("originalFilename", filename.trim());
            }
        }
        if let Some(size) = request.file_size_bytes {
            updates.insert("fileSizeBytes", size as i64);
        }
        if let Some(meta) = request.asset_metadata {
            if let Ok(doc) = mongodb::bson::to_document(&meta) {
                updates.insert("assetMetadata", doc);
            }
        }
        if let Some(title) = request.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest("title cannot be empty".to_string()));
            }
            if title.len() > 200 {
                return Err(AppError::BadRequest(
                    "title cannot exceed 200 characters".to_string(),
                ));
            }
            updates.insert("title", title.trim());
        }
        if let Some(desc) = request.description {
            if desc.len() > 2000 {
                return Err(AppError::BadRequest(
                    "description cannot exceed 2000 characters".to_string(),
                ));
            }
            updates.insert("description", desc.trim());
        }
        if let Some(tags) = request.tags {
            if tags.len() > 10 {
                return Err(AppError::BadRequest(
                    "cannot have more than 10 tags".to_string(),
                ));
            }
            let tags: Vec<String> = tags.into_iter().map(|t| t.trim().to_string()).collect();
            updates.insert("tags", tags);
        }
        if let Some(links) = request.social_media_links {
            if let Ok(doc) = mongodb::bson::to_document(&links) {
                updates.insert("socialMediaLinks", doc);
            }
        }

        if updates.is_empty() {
            return Ok(Self::to_response(existing));
        }

        let updated = self
            .repo
            .update(moment_id, updates)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        tracing::info!(moment_id = %moment_id, "Moment updated successfully");

        Ok(Self::to_response(updated))
    }

    /// Delete a moment (only if owned by the wallet).
    pub async fn delete_moment(&self, wallet: &str, moment_id: &str) -> Result<(), AppError> {
        tracing::debug!(wallet = %wallet, moment_id = %moment_id, "Deleting moment");

        // Verify ownership
        let existing = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        if existing.player_wallet_address != wallet.trim().to_lowercase() {
            tracing::warn!(wallet = %wallet, owner = %existing.player_wallet_address, "Unauthorized delete attempt");
            return Err(AppError::Forbidden(
                "You can only delete your own moments".to_string(),
            ));
        }

        self.repo
            .delete(moment_id)
            .await
            .map_err(|e| AppError::Internal(e))?;

        tracing::info!(moment_id = %moment_id, "Moment deleted successfully");

        Ok(())
    }

    /// Like a moment once per wallet.
    pub async fn like_moment(
        &self,
        wallet: &str,
        moment_id: &str,
    ) -> Result<LikeMomentResponse, AppError> {
        let wallet = wallet.trim().to_lowercase();

        let existing = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        let like = MomentLikeModel::new(moment_id, &wallet);

        match self.likes_repo.create(like.clone()).await {
            Ok(_) => {}
            Err(CreateMomentLikeError::Duplicate) => {
                return Err(AppError::Conflict(
                    "You have already liked this moment".to_string(),
                ));
            }
            Err(CreateMomentLikeError::Internal(e)) => return Err(AppError::Internal(e)),
        }

        let incremented = self
            .repo
            .increment_num_likes(moment_id, 1)
            .await
            .map_err(AppError::Internal)?;

        if !incremented {
            let _ = self.likes_repo.delete_by_id(&like.id).await;
            return Err(AppError::NotFound("Moment not found".to_string()));
        }

        let updated = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        Ok(LikeMomentResponse {
            moment_id: existing.moment_id,
            num_likes: updated.num_likes,
            message: "Moment liked successfully".to_string(),
        })
    }

    /// Convert MomentModel to MomentResponse.
    fn to_response(moment: MomentModel) -> MomentResponse {
        MomentResponse {
            moment_id: moment.moment_id,
            player_wallet_address: moment.player_wallet_address,
            asset_url: moment.asset_url,
            asset_zg_hash: moment.asset_zg_hash,
            num_likes: moment.num_likes,
            num_comments: moment.num_comments,
            asset_metadata: moment
                .asset_metadata
                .and_then(|d| serde_json::to_value(d).ok()),
            title: moment.title,
            description: moment.description,
            tags: moment.tags,
            social_media_links: moment
                .social_media_links
                .and_then(|d| serde_json::to_value(d).ok()),
            created_at: moment
                .created_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                .unwrap_or_default(),
            updated_at: moment
                .updated_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                .unwrap_or_default(),
            original_filename: moment.original_filename,
            file_size_bytes: moment.file_size_bytes,
        }
    }
}
