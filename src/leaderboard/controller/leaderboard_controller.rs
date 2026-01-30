use crate::leaderboard::{controller::LeaderboardState, model::GlobalLeaderboardModel};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GlobalLeaderboardParams {
    #[serde(default = "default_skip")]
    pub skip: u64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_skip() -> u64 {
    0
}

fn default_limit() -> i64 {
    50
}

pub async fn get_global_leaderboard(
    State(state): State<LeaderboardState>,
    Query(params): Query<GlobalLeaderboardParams>,
) -> Result<Json<Vec<GlobalLeaderboardModel>>, String> {
    match state
        .global_service
        .get_global_leaderboard(params.skip, params.limit)
        .await
    {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e),
    }
}

pub async fn refresh_global_leaderboard(
    State(state): State<LeaderboardState>,
) -> Result<String, String> {
    match state.global_service.refresh_global_leaderboard().await {
        Ok(count) => Ok(format!("Refreshed {} entries.", count)),
        Err(e) => Err(e),
    }
}
