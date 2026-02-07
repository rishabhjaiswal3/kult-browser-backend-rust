use crate::game::service::GameService;
use crate::handler::{ApiResponse, AppError};
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

/// Shared state for game endpoints
#[derive(Clone)]
pub struct GameState {
    pub game_service: GameService,
}

#[derive(Debug, Deserialize)]
pub struct AllGamesQuery {
    pub search: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// GET /api/games/all
pub async fn get_all_games(
    State(state): State<GameState>,
    query: Result<Query<AllGamesQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let Query(params) = match query {
        Ok(q) => q,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).min(100);

    match state
        .game_service
        .get_all_games(params.search, page, page_size)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/games/all-categories
pub async fn get_all_categories(State(state): State<GameState>) -> Response {
    match state.game_service.get_all_categories().await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/games/:identification
pub async fn get_game_by_id(
    State(state): State<GameState>,
    Path(identification): Path<String>,
) -> Response {
    match state
        .game_service
        .get_game_by_identification(&identification)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
