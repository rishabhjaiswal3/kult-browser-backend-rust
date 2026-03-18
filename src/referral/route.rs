// src/referral/route.rs

use crate::middleware::AuthPlayer;
use crate::referral::service::ReferralService;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<Arc<ReferralService>> {
    Router::new().route("/me", get(get_my_referral_code))
}

/// Protected Route - Gets or generates the referral link for the logged-in player
#[utoipa::path(
    get,
    path = "/api/referral/me",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Get or create the authenticated player's referral link", body = crate::openapi::ReferralLinkResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Referral link generation failed", body = crate::openapi::ReferralErrorResponse)
    ),
    tag = "Referral"
)]
pub(crate) async fn get_my_referral_code(
    auth_player: AuthPlayer,
    State(service): State<Arc<ReferralService>>,
) -> impl IntoResponse {
    let wallet = auth_player.wallet_address;

    match service.get_or_create_code(&wallet).await {
        Ok(code) => {
            let full_link = format!("https://klt.gm/r/{}", code);
            (
                StatusCode::OK,
                Json(json!({
                    "code": code,
                    "link": full_link
                })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to fetch referral link: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Could not generate referral link" })),
            )
        }
    }
}
