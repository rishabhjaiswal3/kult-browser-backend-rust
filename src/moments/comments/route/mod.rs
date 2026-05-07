use axum::{routing::post, Router};
use mongodb::Database;

use crate::moments::comments::controller::{
    create_comment, create_reply, delete_comment, list_comments, list_replies, update_comment,
    CommentsState,
};
use crate::moments::comments::repository::CommentsRepository;
use crate::moments::comments::service::CommentsService;
use crate::moments::da_events::MomentDAEventRepository;
use crate::moments::MomentsRepository;
use crate::onchain::{OnchainActivityRepository, OnchainActivityService};

pub fn routes(db: Database) -> Router {
    let comments_repository = CommentsRepository::new(&db);
    let moments_repository = MomentsRepository::new(&db);
    let onchain_activity_service = Some(OnchainActivityService::new(
        OnchainActivityRepository::new(&db),
    ));
    let da_event_repo = MomentDAEventRepository::new(&db);
    let service = CommentsService::new(
        comments_repository,
        moments_repository,
        onchain_activity_service,
    )
    .with_da_events(da_event_repo);
    let state = CommentsState { service };

    Router::new()
        .route(
            "/:moment_id/comments",
            post(create_comment).get(list_comments),
        )
        .route(
            "/comments/:comment_id/replies",
            post(create_reply).get(list_replies),
        )
        .route(
            "/comments/:comment_id",
            axum::routing::patch(update_comment).delete(delete_comment),
        )
        .with_state(state)
}
