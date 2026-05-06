// src/moments/controller/moments_controller.rs

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;
use crate::moments::dto::{CreateMomentRequest, UpdateMomentRequest};
use crate::moments::service::MomentsService;

/// Shared state for moments endpoints
#[derive(Clone)]
pub struct MomentsState {
    pub service: MomentsService,
}

/// Query parameters for feed pagination
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
pub struct FeedQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub tags: Option<String>, // Comma-separated tags
    #[serde(rename = "search-query", alias = "searchQuery")]
    #[param(rename = "search-query")]
    pub search_query: Option<String>,
}

/// Query parameters for the authenticated player's moments.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
pub struct MyMomentsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// POST /api/moments
/// Create a new moment (auth required)
#[utoipa::path(
    post,
    path = "/api/moments/register",
    summary = "Create a moment",
    description = "Creates a new moment for the authenticated player.",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateMomentRequest,
    responses(
        (status = 200, description = "Created a new moment", body = crate::openapi::CreateMomentApiResponse),
        (status = 400, description = "Invalid moment payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn create_moment(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    payload: Result<Json<CreateMomentRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .service
        .create_moment(&auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments
/// Get public feed of moments (no auth required)
#[utoipa::path(
    get,
    path = "/api/moments",
    summary = "List public moments",
    description = "Returns the public moments feed with optional tag and search filtering.",
    params(FeedQuery),
    responses(
        (status = 200, description = "Public moments feed", body = crate::openapi::MomentListApiResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn get_feed(
    State(state): State<MomentsState>,
    Query(query): Query<FeedQuery>,
) -> Response {
    let tags = query.tags.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });

    match state
        .service
        .get_feed(query.page, query.per_page, tags, query.search_query)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments/my
/// Get logged-in player's moments (auth required)
#[utoipa::path(
    get,
    path = "/api/moments/my",
    summary = "List my moments",
    description = "Returns the authenticated player's moments.",
    security(
        ("bearer_auth" = [])
    ),
    params(MyMomentsQuery),
    responses(
        (status = 200, description = "Authenticated player's moments", body = crate::openapi::MomentListApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn get_my_moments(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Query(query): Query<MyMomentsQuery>,
) -> Response {
    let page = query.page.unwrap_or_else(default_page);
    let per_page = query.per_page.unwrap_or_else(default_per_page);

    match state
        .service
        .get_player_moments(&auth.wallet_address, page, per_page)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments/:moment_id
/// Get a single moment by ID (no auth required)
#[utoipa::path(
    get,
    path = "/api/moments/{moment_id}",
    summary = "Get a moment",
    description = "Fetches a single moment by its shareable moment ID.",
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    responses(
        (status = 200, description = "Single moment", body = crate::openapi::MomentApiResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn get_moment(
    State(state): State<MomentsState>,
    Path(moment_id): Path<String>,
) -> Response {
    match state.service.get_moment(&moment_id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments/:moment_id/zg-proof
/// Get public 0G storage proof for a moment.
#[utoipa::path(
    get,
    path = "/api/moments/{moment_id}/zg-proof",
    summary = "Get moment 0G proof",
    description = "Returns the 0G storage hashes, tx hashes, gateway links, and migration status for a moment.",
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    responses(
        (status = 200, description = "Moment 0G proof", body = crate::openapi::MomentZgProofApiResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn get_zg_proof(
    State(state): State<MomentsState>,
    Path(moment_id): Path<String>,
) -> Response {
    match state.service.get_zg_proof(&moment_id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /api/moments/:moment_id/zg/retry
/// Requeue 0G storage migration for a moment owned by the authenticated player.
#[utoipa::path(
    post,
    path = "/api/moments/{moment_id}/zg/retry",
    summary = "Retry moment 0G migration",
    description = "Requeues the moment asset and metadata for 0G storage migration.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    responses(
        (status = 200, description = "0G migration retry queued", body = crate::openapi::RetryZgMigrationApiResponse),
        (status = 400, description = "Moment cannot be retried", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Not the moment owner", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn retry_zg_migration(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Path(moment_id): Path<String>,
) -> Response {
    match state
        .service
        .retry_zg_migration(&auth.wallet_address, &moment_id)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// PATCH /api/moments/:moment_id
/// Update a moment (auth required, must own the moment)
#[utoipa::path(
    patch,
    path = "/api/moments/{moment_id}",
    summary = "Update a moment",
    description = "Updates an existing moment owned by the authenticated player.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    request_body = UpdateMomentRequest,
    responses(
        (status = 200, description = "Updated moment", body = crate::openapi::MomentApiResponse),
        (status = 400, description = "Invalid update payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn update_moment(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Path(moment_id): Path<String>,
    payload: Result<Json<UpdateMomentRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .service
        .update_moment(&auth.wallet_address, &moment_id, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// DELETE /api/moments/:moment_id
/// Delete a moment (auth required, must own the moment)
#[utoipa::path(
    delete,
    path = "/api/moments/{moment_id}",
    summary = "Delete a moment",
    description = "Deletes a moment owned by the authenticated player.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    responses(
        (status = 200, description = "Deleted moment", body = crate::openapi::DeleteMomentApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn delete_moment(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Path(moment_id): Path<String>,
) -> Response {
    match state
        .service
        .delete_moment(&auth.wallet_address, &moment_id)
        .await
    {
        Ok(_) => ApiResponse::success(serde_json::json!({
            "message": "Moment deleted successfully"
        }))
        .into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /api/moments/:moment_id/like
/// Like a moment once (auth required)
#[utoipa::path(
    post,
    path = "/api/moments/{moment_id}/like",
    summary = "Like a moment",
    description = "Registers a like for the authenticated player. A player can like a moment only once.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    responses(
        (status = 200, description = "Moment liked", body = crate::openapi::LikeMomentApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 409, description = "Moment already liked by the authenticated player", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments"
)]
pub async fn like_moment(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Path(moment_id): Path<String>,
) -> Response {
    match state
        .service
        .like_moment(&auth.wallet_address, &moment_id)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
