// src/moments/service/moments_service.rs

use crate::external::digital_ocean::spaces::SpacesService;
use crate::game::repository::GameModelRepository;
use crate::handler::AppError;
use crate::moments::da_events::{
    MomentDAEventModel, MomentDAEventRepository, EVENT_MOMENT_CREATED, EVENT_MOMENT_LIKED,
    EVENT_MOMENT_SHARED,
};
use crate::moments::dto::{
    ComputeProofResponse, CreateMomentRequest, CreateMomentResponse, DaProofResponse,
    DaPipelineStatus, LikeMomentResponse, MomentListResponse, MomentPipelineResponse,
    MomentProofResponse, MomentResponse, MomentZgProofResponse, PipelineStageStatus,
    RetryZgMigrationResponse, ShareMomentResponse, StorageProofResponse, UpdateMomentRequest,
};
use crate::moments::model::{MomentLikeModel, MomentModel};
use crate::moments::repository::{CreateMomentLikeError, MomentLikesRepository, MomentsRepository};
use crate::moments::worker::migration_worker::MigrationJob;
use crate::onchain::{metadata_hash, ActivityType, OnchainActivityService, RecordActivityInput};
use crate::redis::ValkyQueue;
use std::collections::HashSet;

const MAX_RELATED_GAMES: usize = 5;

/// Service layer for Moment operations.
#[derive(Clone)]
pub struct MomentsService {
    repo: MomentsRepository,
    likes_repo: MomentLikesRepository,
    games_repo: GameModelRepository,
    queue: Option<ValkyQueue>,
    spaces_service: SpacesService,
    onchain_activity_service: Option<OnchainActivityService>,
    da_event_repo: Option<MomentDAEventRepository>,
}

impl MomentsService {
    pub fn new(
        repo: MomentsRepository,
        likes_repo: MomentLikesRepository,
        games_repo: GameModelRepository,
        spaces_service: SpacesService,
        onchain_activity_service: Option<OnchainActivityService>,
    ) -> Self {
        Self {
            repo,
            likes_repo,
            games_repo,
            queue: None,
            spaces_service,
            onchain_activity_service,
            da_event_repo: None,
        }
    }

    /// Create with queue for migration support.
    pub fn with_queue(
        repo: MomentsRepository,
        likes_repo: MomentLikesRepository,
        games_repo: GameModelRepository,
        queue: ValkyQueue,
        spaces_service: SpacesService,
        onchain_activity_service: Option<OnchainActivityService>,
    ) -> Self {
        Self {
            repo,
            likes_repo,
            games_repo,
            queue: Some(queue),
            spaces_service,
            onchain_activity_service,
            da_event_repo: None,
        }
    }

    pub fn with_da_events(mut self, da_event_repo: MomentDAEventRepository) -> Self {
        self.da_event_repo = Some(da_event_repo);
        self
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
        let related_games = self.normalize_related_games(&request.related_games).await?;
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
        let onchain_metadata = serde_json::json!({
            "title": request.title.trim(),
            "tags": request.tags,
            "relatedGames": request.related_games,
            "assetUrl": request.asset_url,
        });

        let moment = MomentModel {
            id: None,
            moment_id: moment_id.clone(),
            player_wallet_address: wallet.trim().to_string(),
            asset_url: request
                .asset_url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(|u| u.to_string()),
            asset_zg_hash: None,
            metadata_zg_hash: None,
            zg_status: request
                .asset_url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(|_| "pending".to_string()),
            asset_zg_tx_hash: None,
            metadata_zg_tx_hash: None,
            zg_error: None,
            zg_uploaded_at: None,
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
            related_games,
            social_media_links: request
                .social_media_links
                .and_then(|v| mongodb::bson::to_document(&v).ok()),
            created_at: None,
            updated_at: None,
            original_filename: None,
            file_size_bytes: None,
            ai_caption: None,
            ai_rank_score: None,
            ai_highlights: vec![],
            ai_status: None,
            ai_moment_type: None,
            ai_skill_score: None,
            ai_reaction_quality: None,
            ai_rarity: None,
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
                    asset_zg_hash: None,
                    asset_zg_tx_hash: None,
                    metadata_zg_hash: None,
                    metadata_zg_tx_hash: None,
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
        self.enqueue_onchain_activity(
            wallet,
            ActivityType::MomentCreated,
            &moment_id,
            &moment_id,
            &onchain_metadata,
        )
        .await;

        self.enqueue_da_event(
            &moment_id,
            EVENT_MOMENT_CREATED,
            wallet,
            serde_json::json!({ "title": request.title.trim() }),
        )
        .await;

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

    /// Get the public 0G proof document for a moment.
    pub async fn get_zg_proof(&self, moment_id: &str) -> Result<MomentZgProofResponse, AppError> {
        let moment = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        Ok(Self::to_zg_proof(moment))
    }

    /// Requeue a moment for 0G storage migration. Owner-only.
    pub async fn retry_zg_migration(
        &self,
        wallet: &str,
        moment_id: &str,
    ) -> Result<RetryZgMigrationResponse, AppError> {
        let moment = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        if !moment.player_wallet_address.eq_ignore_ascii_case(wallet) {
            return Err(AppError::Forbidden(
                "You can retry migration only for your own moment".to_string(),
            ));
        }

        let asset_url = moment
            .asset_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("Moment has no asset URL to migrate".to_string())
            })?;

        let asset_type = moment
            .asset_metadata
            .as_ref()
            .and_then(|doc| doc.get_str("fileType").ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let queue = self.queue.as_ref().ok_or_else(|| {
            AppError::BadRequest("0G migration queue is not available".to_string())
        })?;

        let job = MigrationJob {
            asset_url: asset_url.to_string(),
            asset_id: moment.moment_id.clone(),
            asset_type,
            asset_zg_hash: moment.asset_zg_hash.clone(),
            asset_zg_tx_hash: moment.asset_zg_tx_hash.clone(),
            metadata_zg_hash: moment.metadata_zg_hash.clone(),
            metadata_zg_tx_hash: moment.metadata_zg_tx_hash.clone(),
            attempt: 1,
        };

        queue.push_async(&job).await.map_err(|e| {
            tracing::error!(moment_id = %moment.moment_id, error = %e, "Failed to requeue 0G migration");
            AppError::Internal(format!("Failed to queue 0G migration retry: {}", e))
        })?;

        self.repo
            .mark_zg_pending(&moment.moment_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(RetryZgMigrationResponse {
            moment_id: moment.moment_id,
            zg_status: "pending".to_string(),
            message: "0G migration retry queued".to_string(),
        })
    }

    /// Get public feed of moments.
    pub async fn get_feed(
        &self,
        page: u32,
        per_page: u32,
        tags: Option<Vec<String>>,
        search_query: Option<String>,
    ) -> Result<MomentListResponse, AppError> {
        let page = if page == 0 { 1 } else { page };
        let per_page = per_page.min(50).max(1);
        let search_query = search_query
            .map(|q| q.trim().to_string())
            .filter(|q| !q.is_empty());

        tracing::debug!(
            page = page,
            per_page = per_page,
            search_query = search_query.as_deref().unwrap_or(""),
            "Fetching moments feed"
        );

        let moments = self
            .repo
            .find_all(page, per_page, tags.clone(), search_query.clone())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch feed");
                AppError::Internal(e)
            })?;

        let total = self.repo.count_all(tags, search_query).await.map_err(|e| {
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
        let wallet = wallet.trim().to_string();
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

        if existing.player_wallet_address != wallet.trim() {
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
        if let Some(related_games) = request.related_games {
            let related_games = self.normalize_related_games(&related_games).await?;
            updates.insert("relatedGames", related_games);
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

        if existing.player_wallet_address != wallet.trim() {
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
        let wallet = wallet.trim().to_string();

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

        self.enqueue_onchain_activity(
            &wallet,
            ActivityType::MomentLiked,
            &existing.moment_id,
            &format!("{}:{}", existing.moment_id, wallet),
            &serde_json::json!({ "numLikes": updated.num_likes }),
        )
        .await;

        self.enqueue_da_event(
            moment_id,
            EVENT_MOMENT_LIKED,
            &wallet,
            serde_json::json!({ "numLikes": updated.num_likes }),
        )
        .await;

        Ok(LikeMomentResponse {
            moment_id: existing.moment_id,
            num_likes: updated.num_likes,
            message: "Moment liked successfully".to_string(),
        })
    }

    /// Record a share event on 0G DA.
    pub async fn share_moment(
        &self,
        wallet: &str,
        moment_id: &str,
    ) -> Result<ShareMomentResponse, AppError> {
        let _ = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        self.enqueue_da_event(
            moment_id,
            EVENT_MOMENT_SHARED,
            wallet,
            serde_json::json!({ "sharedBy": wallet }),
        )
        .await;

        Ok(ShareMomentResponse {
            moment_id: moment_id.to_string(),
            message: "Share recorded on 0G DA".to_string(),
        })
    }

    /// Return DA events for a moment.
    pub async fn get_da_events(
        &self,
        moment_id: &str,
    ) -> Result<Vec<crate::moments::dto::MomentDAEventResponse>, AppError> {
        let Some(repo) = &self.da_event_repo else {
            return Ok(vec![]);
        };

        let events = repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(events
            .into_iter()
            .map(|e| crate::moments::dto::MomentDAEventResponse {
                event_type: e.event_type,
                actor_wallet: e.actor_wallet,
                da_status: e.da_status,
                da_request_id: e.da_request_id,
                da_batch_id: e.da_batch_id,
                da_blob_index: e.da_blob_index,
                da_batch_header_hash: e.da_batch_header_hash,
                da_confirmation_block: e.da_confirmation_block,
                da_finalized_at: e.da_finalized_at,
                da_error: e.da_error,
                created_at: e
                    .created_at
                    .and_then(|dt| dt.try_to_rfc3339_string().ok()),
            })
            .collect())
    }

    /// Assemble a full proof bundle for a moment: storage + DA + compute.
    pub async fn get_proof(&self, moment_id: &str) -> Result<MomentProofResponse, AppError> {
        let moment = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        let storage_verified = moment.asset_zg_hash.is_some() && moment.metadata_zg_hash.is_some();
        let storage = StorageProofResponse {
            status: moment.zg_status.clone().unwrap_or_else(|| "none".to_string()),
            asset_hash: moment.asset_zg_hash.clone(),
            metadata_hash: moment.metadata_zg_hash.clone(),
            asset_url: moment
                .asset_zg_hash
                .as_deref()
                .and_then(|h| crate::config::CONFIG.zg.gateway_url_for_hash(h)),
            metadata_url: moment
                .metadata_zg_hash
                .as_deref()
                .and_then(|h| crate::config::CONFIG.zg.gateway_url_for_hash(h)),
            asset_tx_hash: moment.asset_zg_tx_hash.clone(),
            metadata_tx_hash: moment.metadata_zg_tx_hash.clone(),
            uploaded_at: moment
                .zg_uploaded_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default()),
            verified: storage_verified,
        };

        let da_events = self.get_da_events(moment_id).await?;
        let finalized_count = da_events.iter().filter(|e| e.da_status == "finalized").count();
        let da = DaProofResponse {
            total_events: da_events.len(),
            finalized_events: finalized_count,
            events: da_events,
        };

        let compute = ComputeProofResponse {
            status: moment.ai_status.clone().unwrap_or_else(|| "none".to_string()),
            caption: moment.ai_caption.clone(),
            rank_score: moment.ai_rank_score,
            moment_type: moment.ai_moment_type.clone(),
            skill_score: moment.ai_skill_score,
            rarity: moment.ai_rarity.clone(),
        };

        Ok(MomentProofResponse {
            moment_id: moment_id.to_string(),
            storage,
            da,
            compute,
        })
    }

    /// Per-layer pipeline status for a moment.
    pub async fn get_pipeline(&self, moment_id: &str) -> Result<MomentPipelineResponse, AppError> {
        let moment = self
            .repo
            .find_by_moment_id(moment_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Moment not found".to_string()))?;

        let storage = PipelineStageStatus {
            status: moment.zg_status.clone().unwrap_or_else(|| "none".to_string()),
            detail: moment.zg_error.clone(),
        };

        let da_events = self.get_da_events(moment_id).await?;
        let total = da_events.len();
        let finalized = da_events.iter().filter(|e| e.da_status == "finalized").count();
        let dispersing = da_events.iter().filter(|e| e.da_status == "dispersing").count();
        let pending = da_events.iter().filter(|e| e.da_status == "pending").count();
        let failed = da_events.iter().filter(|e| e.da_status == "failed").count();
        let da = DaPipelineStatus { total, finalized, dispersing, pending, failed };

        let compute = PipelineStageStatus {
            status: moment.ai_status.clone().unwrap_or_else(|| "none".to_string()),
            detail: None,
        };

        Ok(MomentPipelineResponse {
            moment_id: moment_id.to_string(),
            storage,
            da,
            compute,
        })
    }

    /// Convert MomentModel to MomentResponse.
    fn to_response(moment: MomentModel) -> MomentResponse {
        let asset_zg_url = moment
            .asset_zg_hash
            .as_deref()
            .and_then(|hash| crate::config::CONFIG.zg.gateway_url_for_hash(hash));
        let metadata_zg_url = moment
            .metadata_zg_hash
            .as_deref()
            .and_then(|hash| crate::config::CONFIG.zg.gateway_url_for_hash(hash));
        let asset_zg_tx_url = moment
            .asset_zg_tx_hash
            .as_deref()
            .and_then(|hash| crate::config::CONFIG.zg.explorer_url_for_tx(hash));
        let metadata_zg_tx_url = moment
            .metadata_zg_tx_hash
            .as_deref()
            .and_then(|hash| crate::config::CONFIG.zg.explorer_url_for_tx(hash));

        MomentResponse {
            moment_id: moment.moment_id,
            player_wallet_address: moment.player_wallet_address,
            asset_url: moment.asset_url,
            asset_zg_hash: moment.asset_zg_hash,
            metadata_zg_hash: moment.metadata_zg_hash,
            zg_status: moment.zg_status,
            asset_zg_tx_hash: moment.asset_zg_tx_hash,
            metadata_zg_tx_hash: moment.metadata_zg_tx_hash,
            asset_zg_url,
            metadata_zg_url,
            asset_zg_tx_url,
            metadata_zg_tx_url,
            zg_error: moment.zg_error,
            zg_uploaded_at: moment
                .zg_uploaded_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default()),
            num_likes: moment.num_likes,
            num_comments: moment.num_comments,
            asset_metadata: moment
                .asset_metadata
                .and_then(|d| serde_json::to_value(d).ok()),
            title: moment.title,
            description: moment.description,
            tags: moment.tags,
            related_games: moment.related_games,
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
            ai_caption: moment.ai_caption,
            ai_rank_score: moment.ai_rank_score,
            ai_highlights: moment.ai_highlights,
            ai_status: moment.ai_status,
            ai_moment_type: moment.ai_moment_type,
            ai_skill_score: moment.ai_skill_score,
            ai_reaction_quality: moment.ai_reaction_quality,
            ai_rarity: moment.ai_rarity,
        }
    }

    fn to_zg_proof(moment: MomentModel) -> MomentZgProofResponse {
        let response = Self::to_response(moment);
        MomentZgProofResponse {
            moment_id: response.moment_id,
            zg_status: response.zg_status,
            asset_zg_hash: response.asset_zg_hash,
            metadata_zg_hash: response.metadata_zg_hash,
            asset_zg_tx_hash: response.asset_zg_tx_hash,
            metadata_zg_tx_hash: response.metadata_zg_tx_hash,
            asset_zg_url: response.asset_zg_url,
            metadata_zg_url: response.metadata_zg_url,
            asset_zg_tx_url: response.asset_zg_tx_url,
            metadata_zg_tx_url: response.metadata_zg_tx_url,
            zg_uploaded_at: response.zg_uploaded_at,
            zg_error: response.zg_error,
        }
    }

    async fn normalize_related_games(
        &self,
        related_games: &[String],
    ) -> Result<Vec<String>, AppError> {
        let mut seen = HashSet::new();
        let mut normalized = Vec::new();

        for related_game in related_games {
            let related_game = related_game.trim();

            if related_game.is_empty() {
                return Err(AppError::BadRequest(
                    "related game identification cannot be empty".to_string(),
                ));
            }

            if seen.insert(related_game.to_string()) {
                normalized.push(related_game.to_string());
            }
        }

        if normalized.len() > MAX_RELATED_GAMES {
            return Err(AppError::BadRequest(format!(
                "cannot have more than {} related games",
                MAX_RELATED_GAMES
            )));
        }

        let released_identifications = self
            .games_repo
            .find_released_identifications(&normalized)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to validate related game identifications");
                AppError::Internal(e.to_string())
            })?;

        if released_identifications.len() != normalized.len() {
            let missing: Vec<String> = normalized
                .iter()
                .filter(|identification| !released_identifications.contains(*identification))
                .cloned()
                .collect();

            return Err(AppError::BadRequest(format!(
                "unknown or unreleased related game identification(s): {}",
                missing.join(", ")
            )));
        }

        Ok(normalized)
    }

    async fn enqueue_onchain_activity<T: serde::Serialize>(
        &self,
        wallet: &str,
        activity_type: ActivityType,
        moment_id: &str,
        entity_id: &str,
        metadata: &T,
    ) {
        let Some(service) = &self.onchain_activity_service else {
            return;
        };

        if let Err(e) = service
            .enqueue_activity(RecordActivityInput {
                user_wallet: wallet.trim().to_string(),
                activity_type,
                moment_id: moment_id.to_string(),
                entity_id: entity_id.to_string(),
                metadata_hash: metadata_hash(metadata),
            })
            .await
        {
            tracing::error!(
                error = %e,
                moment_id = %moment_id,
                "Failed to enqueue onchain activity"
            );
        }
    }

    async fn enqueue_da_event(
        &self,
        moment_id: &str,
        event_type: &str,
        actor_wallet: &str,
        event_data: serde_json::Value,
    ) {
        let Some(repo) = &self.da_event_repo else {
            return;
        };

        let event = MomentDAEventModel::new(moment_id, event_type, actor_wallet, event_data);
        if let Err(e) = repo.create(event).await {
            tracing::error!(
                error = %e,
                moment_id = %moment_id,
                event_type = %event_type,
                "Failed to create DA event"
            );
        }
    }
}
