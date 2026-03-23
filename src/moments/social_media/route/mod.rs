// src/moments/social_media/route/mod.rs

use axum::{routing::post, Router};
use mongodb::Database;

use crate::moments::social_media::controller::{submit_post, SocialMediaState};
use crate::moments::social_media::repository::post_repository::PostRepository;
use crate::moments::social_media::service::post_service::PostService;
use crate::moments::MomentsRepository;
use crate::redis::ValkyQueue;

/// Build routes for the social media sub-module.
///
/// Routes:
/// - POST /submit  — Submit a social media post URL for validation (auth required)
///
/// Mounted at: /api/moments/social-media
pub fn routes(db: Database, queue: Option<ValkyQueue>) -> Router {
    let post_repository = PostRepository::new(&db);
    let moments_repository = MomentsRepository::new(&db);
    let post_service = match queue {
        Some(q) => PostService::with_queue(post_repository, moments_repository, q),
        None => PostService::new(post_repository, moments_repository),
    };

    let state = SocialMediaState { post_service };

    Router::new()
        .route("/submit-url", post(submit_post))
        .with_state(state)
}
