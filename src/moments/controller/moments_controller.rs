// src/moments/controller/moments_controller.rs

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub tags: Option<String>, // Comma-separated tags
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// POST /api/moments
/// Create a new moment (auth required)
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
        .get_feed(query.page, query.per_page, tags)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments/my
/// Get logged-in player's moments (auth required)
pub async fn get_my_moments(
    State(state): State<MomentsState>,
    auth: AuthPlayer,
    Query(query): Query<FeedQuery>,
) -> Response {
    match state
        .service
        .get_player_moments(&auth.wallet_address, query.page, query.per_page)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/moments/:moment_id
/// Get a single moment by ID (no auth required)
pub async fn get_moment(
    State(state): State<MomentsState>,
    Path(moment_id): Path<String>,
) -> Response {
    match state.service.get_moment(&moment_id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// PATCH /api/moments/:moment_id
/// Update a moment (auth required, must own the moment)
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
