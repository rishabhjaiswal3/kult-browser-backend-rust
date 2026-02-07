use crate::handler::{ApiResponse, AppError};
use crate::leaderboard::controller::LeaderboardState;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
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

/// GET /api/leaderboard/game/:identification
pub async fn get_game_leaderboard(
    State(state): State<LeaderboardState>,
    Path(identification): Path<String>,
    query: Result<Query<LeaderboardParams>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let Query(params) = match query {
        Ok(q) => q,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    let page = params.page.max(1);
    let page_size = params.page_size.min(100);

    match state
        .game_service
        .fetch_leaderboard_paginated(&identification, page, page_size)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
