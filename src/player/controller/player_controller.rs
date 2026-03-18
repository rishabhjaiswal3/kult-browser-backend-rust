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
#[utoipa::path(
    post,
    path = "/api/player/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login or create a player", body = crate::openapi::LoginApiResponse),
        (status = 400, description = "Invalid login payload", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Player"
)]
pub async fn login(
    State(state): State<PlayerState>,
    headers: axum::http::header::HeaderMap,
    payload: Result<Json<LoginRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0.0.0.0");

    match state.player_service.login(request, ip_address).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /api/player/profile
#[utoipa::path(
    get,
    path = "/api/player/profile",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Authenticated player profile", body = crate::openapi::PlayerProfileApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Player not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Player"
)]
pub async fn get_profile(State(state): State<PlayerState>, auth: AuthPlayer) -> Response {
    match state.player_service.get_profile(&auth.wallet_address).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// PATCH /api/player/name
#[utoipa::path(
    patch,
    path = "/api/player/name",
    security(
        ("bearer_auth" = [])
    ),
    request_body = UpdateNameRequest,
    responses(
        (status = 200, description = "Updated player display name", body = crate::openapi::UpdateNameApiResponse),
        (status = 400, description = "Invalid update payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Player not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Player"
)]
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
