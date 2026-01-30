use crate::leaderboard::{controller::LeaderboardState, model::LeaderboardEntry};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
    #[serde(default = "default_skip")]
    pub skip: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_skip() -> u32 {
    0
}

fn default_limit() -> u32 {
    50
}

pub async fn get_game_leaderboard(
    State(state): State<LeaderboardState>,
    Path(identification): Path<String>,
    Query(params): Query<super::game_leaderboard_controller::LeaderboardParams>,
) -> Result<Json<Vec<LeaderboardEntry>>, String> {
    // Wait, the service returns Vec<LeaderboardEntry>.
    // Result<Json<Vec<...>>>

    match state
        .game_service
        .fetch_leaderboard(&identification, params.skip, params.limit)
        .await
    {
        Ok(entries) => Ok(Json(entries)), // Error: Json(...) wraps Vec, so return type is Json<Vec>
        Err(e) => Err(e),
    }
}

// Correct signature:
// Result<Json<Vec<LeaderboardEntry>>, String> should be Result<Json<Vec<LeaderboardEntry>>, (StatusCode, String)> generally but
// for MVP String is accepted by Axum as plain text response.
