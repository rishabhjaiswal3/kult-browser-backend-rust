// src/moments/social_media/route/mod.rs

use axum::{routing::post, Router};

use crate::moments::social_media::controller::{submit_post, SocialMediaState};
use crate::moments::social_media::service::post_service::PostService;
use crate::redis::ValkyQueue;

/// Build routes for the social media sub-module.
///
/// Routes:
/// - POST /submit  — Submit a social media post URL for validation (auth required)
///
/// Mounted at: /api/moments/social-media
pub async fn routes(queue: Option<ValkyQueue>) -> Router {
    let post_service = match queue {
        Some(q) => PostService::with_queue(q)
            .await
            .expect("Failed to create PostService with queue"),
        None => PostService::new()
            .await
            .expect("Failed to create PostService"),
    };

    let state = SocialMediaState { post_service };

    Router::new()
        .route("/submit-url", post(submit_post))
        .with_state(state)
}
