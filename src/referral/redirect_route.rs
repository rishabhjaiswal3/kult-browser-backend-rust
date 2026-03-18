// src/referral/redirect_route.rs

use crate::referral::analytics::ClickAnalyticsService;
use crate::referral::service::ReferralService;
use axum::{
    extract::{Path, State},
    http::header::{HeaderMap, HeaderValue, LOCATION, USER_AGENT},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedirectAppState {
    pub referral_service: Arc<ReferralService>,
    pub click_analytics: Arc<ClickAnalyticsService>,
}

pub fn router() -> Router<RedirectAppState> {
    Router::new().route("/:code", get(handle_redirect))
}

/// Public Route - Intercepts the click, logs to queue, and 302 Redirects to frontend
#[utoipa::path(
    get,
    path = "/r/{code}",
    params(
        ("code" = String, Path, description = "Referral code")
    ),
    responses(
        (
            status = 302,
            description = "Redirect to the referral landing page",
            headers(
                ("Location" = String, description = "Resolved redirect target")
            )
        )
    ),
    tag = "Referral"
)]
pub(crate) async fn handle_redirect(
    Path(code): Path<String>,
    State(state): State<RedirectAppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Extract Analytics Data
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0.0.0.0");

    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // 2. Fire and forget click logging
    state.click_analytics.log_click(&code, ip, user_agent).await;

    // 3. Prepare the redirect destination
    let frontend_url = format!("https://kult.games/join?ref={}", code);

    // 4. Return HTTP 302 Found
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        LOCATION,
        HeaderValue::from_str(&frontend_url)
            .unwrap_or(HeaderValue::from_static("https://kult.games/join")),
    );

    // TODO: In a production SSR environment, this could return an HTML body
    // with `<meta property="og:title">` tags *along* with the 302 status code
    // to force WhatsApp to scrape an image card.

    (axum::http::StatusCode::FOUND, response_headers, "")
}
