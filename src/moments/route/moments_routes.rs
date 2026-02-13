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
use crate::external::digital_ocean::spaces::SpacesService;

pub fn routes(db: Database, queue: Option<ValkyQueue>) -> Router {
    let repo = MomentsRepository::new(&db);
    let spaces_service = SpacesService::new();

    let service = match queue {
        Some(q) => MomentsService::with_queue(repo, q, spaces_service),
        None => MomentsService::new(repo, spaces_service),
    };
    let state = MomentsState { service };

    Router::new()
        .route("/register", post(create_moment))
        .route("/", get(get_feed))
        .route("/my", get(get_my_moments))
        .route(
            "/:moment_id",
            get(get_moment).patch(update_moment).delete(delete_moment),
        )
        .with_state(state)
}
