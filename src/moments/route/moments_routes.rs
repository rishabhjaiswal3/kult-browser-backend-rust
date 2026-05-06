// src/moments/route/moments_routes.rs

use axum::{
    routing::{get, post},
    Router,
};
use mongodb::Database;

use crate::game::repository::GameModelRepository;
use crate::moments::comments;
use crate::moments::controller::{
    create_moment, delete_moment, get_feed, get_moment, get_my_moments, get_zg_proof, like_moment,
    retry_zg_migration, update_moment, MomentsState,
};
use crate::moments::creators;
use crate::moments::repository::{MomentLikesRepository, MomentsRepository};
use crate::moments::service::MomentsService;
use crate::onchain::{OnchainActivityRepository, OnchainActivityService};
use crate::redis::ValkyQueue;

/// Build routes for the moments module.
///
/// Routes:
/// - POST   /register    - Create/register a new moment (auth required)
/// - GET    /            - Get public feed (paginated, tag/search filtered)
/// - GET    /my          - Get player's moments (auth required)
/// - GET    /:moment_id  - Get single moment
/// - PATCH  /:moment_id  - Update moment (auth, owner only)
/// - DELETE /:moment_id  - Delete moment (auth, owner only)
use crate::external::digital_ocean::spaces::SpacesService;

pub fn routes(db: Database, queue: Option<ValkyQueue>) -> Router {
    let repo = MomentsRepository::new(&db);
    let likes_repo = MomentLikesRepository::new(&db);
    let games_repo = GameModelRepository::new(&db);
    let spaces_service = SpacesService::new();
    let onchain_activity_service = Some(OnchainActivityService::new(
        OnchainActivityRepository::new(&db),
    ));
    let comments_router = comments::route::routes(db.clone());
    let creators_router = creators::route::routes(db.clone());

    let service = match queue {
        Some(q) => MomentsService::with_queue(
            repo,
            likes_repo,
            games_repo,
            q,
            spaces_service,
            onchain_activity_service,
        ),
        None => MomentsService::new(
            repo,
            likes_repo,
            games_repo,
            spaces_service,
            onchain_activity_service,
        ),
    };
    let state = MomentsState { service };

    Router::new()
        .route("/register", post(create_moment))
        .route("/", get(get_feed))
        .route("/my", get(get_my_moments))
        .route("/:moment_id/like", post(like_moment))
        .route("/:moment_id/zg-proof", get(get_zg_proof))
        .route("/:moment_id/zg/retry", post(retry_zg_migration))
        .route(
            "/:moment_id",
            get(get_moment).patch(update_moment).delete(delete_moment),
        )
        .with_state(state)
        .merge(creators_router)
        .merge(comments_router)
}
