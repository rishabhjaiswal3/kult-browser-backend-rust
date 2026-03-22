use crate::admin::service::GameLeaderboardConfigAdminService;
use crate::handler::{ApiResponse, AppError};
use crate::leaderboard::model::GameLeaderboardConfig;
use crate::leaderboard::repository::GameLeaderboardConfigRepository;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Json, Router,
};
use mongodb::Database;

#[derive(Clone)]
struct GameLeaderboardConfigAdminState {
    service: GameLeaderboardConfigAdminService,
}

pub fn routes(db: Database) -> Router {
    let repo = GameLeaderboardConfigRepository::new(&db);
    let state = GameLeaderboardConfigAdminState {
        service: GameLeaderboardConfigAdminService::new(repo),
    };

    Router::new()
        .route(
            "/game-leaderboard-config",
            post(insert_game_leaderboard_config),
        )
        .route(
            "/game-leaderboard-config/:identification",
            delete(delete_game_leaderboard_config),
        )
        .with_state(state)
}

async fn insert_game_leaderboard_config(
    State(state): State<GameLeaderboardConfigAdminState>,
    payload: Result<Json<GameLeaderboardConfig>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(config) = match payload {
        Ok(payload) => payload,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state.service.insert_config(config).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn delete_game_leaderboard_config(
    State(state): State<GameLeaderboardConfigAdminState>,
    Path(identification): Path<String>,
) -> Response {
    match state.service.delete_config(&identification).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(error) => error.into_response(),
    }
}
