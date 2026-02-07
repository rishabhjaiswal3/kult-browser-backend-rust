use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;
use crate::player::dto::{LoginRequest, UpdateNameRequest};
use crate::player::PlayerService;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Shared state for player endpoints
#[derive(Clone)]
pub struct PlayerState {
    pub player_service: PlayerService,
}

/// POST /api/player/login
pub async fn login(
    State(state): State<PlayerState>,
    payload: Result<Json<LoginRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state.player_service.login(request).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/player/profile
pub async fn get_profile(State(state): State<PlayerState>, auth: AuthPlayer) -> Response {
    match state.player_service.get_profile(&auth.wallet_address).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// PATCH /api/player/name
pub async fn update_name(
    State(state): State<PlayerState>,
    auth: AuthPlayer,
    payload: Result<Json<UpdateNameRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .player_service
        .update_name(&auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
