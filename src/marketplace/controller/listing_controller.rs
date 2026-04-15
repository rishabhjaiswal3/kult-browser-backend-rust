// src/marketplace/controller/listing_controller.rs

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::handler::ApiResponse;
use crate::marketplace::service::ListingService;

/// Shared state for marketplace listing endpoints.
#[derive(Clone)]
pub struct MarketplaceState {
    pub listing_service: ListingService,
}

/// Query parameters for listing browsing.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListingsQuery {
    pub game_identification: Option<String>,
    pub asset_type: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// GET /marketplace
#[utoipa::path(
    get,
    path = "/api/marketplace",
    summary = "List active marketplace listings",
    description = "Returns paginated active listings with optional game and asset type filters.",
    params(ListingsQuery),
    responses(
        (status = 200, description = "Paginated listings", body = crate::openapi::ListingListApiResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace"
)]
pub async fn get_listings(
    State(state): State<MarketplaceState>,
    Query(query): Query<ListingsQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state
        .listing_service
        .get_listings(query.game_identification, query.asset_type, page, per_page)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /marketplace/:id
#[utoipa::path(
    get,
    path = "/api/marketplace/{id}",
    summary = "Get a marketplace listing",
    description = "Fetches a single marketplace listing by its ID.",
    params(
        ("id" = String, Path, description = "Listing ID")
    ),
    responses(
        (status = 200, description = "Single listing", body = crate::openapi::ListingApiResponse),
        (status = 404, description = "Listing not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace"
)]
pub async fn get_listing(
    State(state): State<MarketplaceState>,
    Path(id): Path<String>,
) -> Response {
    match state.listing_service.get_listing(&id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
