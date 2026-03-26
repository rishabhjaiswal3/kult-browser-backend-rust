use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::handler::ApiResponse;
use crate::middleware::AuthPlayer;
use crate::moments::creators::service::CreatorsService;

#[derive(Clone)]
pub struct CreatorsState {
    pub service: CreatorsService,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
pub struct LeaderboardQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

#[utoipa::path(
    get,
    path = "/api/moments/creators/me",
    summary = "Get my creator profile",
    description = "Returns the authenticated player's moments creator profile and score breakdown.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Creator profile", body = crate::openapi::CreatorProfileApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Creator profile not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments Creators"
)]
pub async fn get_me(State(state): State<CreatorsState>, auth: AuthPlayer) -> Response {
    match state.service.get_me(&auth.wallet_address).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/moments/creators/leaderboard",
    summary = "Get creators leaderboard",
    description = "Returns the moments creators leaderboard ranked by total score.",
    params(LeaderboardQuery),
    responses(
        (status = 200, description = "Creators leaderboard", body = crate::openapi::CreatorLeaderboardApiResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Moments Creators"
)]
pub async fn get_leaderboard(
    State(state): State<CreatorsState>,
    Query(query): Query<LeaderboardQuery>,
) -> Response {
    let page = query.page.unwrap_or_else(default_page);
    let page_size = query.page_size.unwrap_or_else(default_page_size);

    match state.service.get_leaderboard(page, page_size).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
