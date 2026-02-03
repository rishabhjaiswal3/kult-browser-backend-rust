use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::middleware::AuthPlayer;
use crate::player::dto::{LoginRequest, UpdateNameRequest};
use crate::player::PlayerService;

/// Shared state for player endpoints
#[derive(Clone)]
pub struct PlayerState {
    pub player_service: PlayerService,
}

/// POST /api/player/login
///
/// Login or register a player with their wallet address.
/// No authentication required.
pub async fn login(
    State(state): State<PlayerState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.player_service.login(payload).await {
        Ok(response) => (StatusCode::OK, Json(json!(response))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e })),
        ),
    }
}

/// GET /api/player/profile
///
/// Get the authenticated player's profile with aggregated stats.
/// Requires JWT authentication.
pub async fn get_profile(State(state): State<PlayerState>, auth: AuthPlayer) -> impl IntoResponse {
    match state.player_service.get_profile(&auth.wallet_address).await {
        Ok(response) => (StatusCode::OK, Json(json!(response))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": e })),
        ),
    }
}

/// PATCH /api/player/name
///
/// Update the authenticated player's display name.
/// Requires JWT authentication.
pub async fn update_name(
    State(state): State<PlayerState>,
    auth: AuthPlayer,
    Json(payload): Json<UpdateNameRequest>,
) -> impl IntoResponse {
    match state
        .player_service
        .update_name(&auth.wallet_address, payload)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(json!(response))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e })),
        ),
    }
}
