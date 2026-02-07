use crate::handler::{ApiResponse, AppError};
use crate::leaderboard::controller::LeaderboardState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GlobalLeaderboardParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    50
}

/// GET /api/leaderboard/global
pub async fn get_global_leaderboard(
    State(state): State<LeaderboardState>,
    query: Result<Query<GlobalLeaderboardParams>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let Query(params) = match query {
        Ok(q) => q,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    let page = params.page.max(1);
    let page_size = params.page_size.min(100);

    match state
        .global_service
        .get_global_leaderboard_paginated(page, page_size)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /api/leaderboard/refresh
pub async fn refresh_global_leaderboard(State(state): State<LeaderboardState>) -> Response {
    match state.global_service.refresh_global_leaderboard().await {
        Ok(count) => ApiResponse::success(serde_json::json!({
            "refreshed": count,
            "message": format!("Refreshed {} entries", count)
        }))
        .into_response(),
        Err(e) => e.into_response(),
    }
}
