// src/moments/route/moments_routes.rs

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use mongodb::Database;

use crate::moments::controller::{
    create_moment, delete_moment, get_feed, get_moment, get_my_moments, update_moment, MomentsState,
};
use crate::moments::repository::MomentsRepository;
use crate::moments::service::MomentsService;
use crate::redis::ValkyQueue;

/// Build routes for the moments module.
///
/// Routes:
/// - POST   /register    - Create/register a new moment (auth required)
/// - GET    /            - Get public feed (paginated)
/// - GET    /my          - Get player's moments (auth required)
/// - GET    /:moment_id  - Get single moment
/// - PATCH  /:moment_id  - Update moment (auth, owner only)
/// - DELETE /:moment_id  - Delete moment (auth, owner only)
pub fn routes(db: Database, queue: Option<ValkyQueue>) -> Router {
    let repo = MomentsRepository::new(&db);
    let service = match queue {
        Some(q) => MomentsService::with_queue(repo, q),
        None => MomentsService::new(repo),
    };
    let state = MomentsState { service };

    Router::new()
        .route("/register", post(create_moment))
        .route("/", get(get_feed))
        .route("/my", get(get_my_moments))
        .route("/{moment_id}", get(get_moment))
        .route("/{moment_id}", patch(update_moment))
        .route("/{moment_id}", delete(delete_moment))
        .with_state(state)
}
