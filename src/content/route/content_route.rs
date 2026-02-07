use axum::{routing::get, Router};
use mongodb::Database;
use std::sync::Arc;

use crate::content::controller::{get_content, ContentState};
use crate::content::repository::ContentConfigRepository;
use crate::game::repository::GameModelRepository;

/// Build content module routes
///
/// Creates the router for all content-related endpoints and initializes
/// required repositories and state.
pub fn routes(db: Database) -> Router {
    let config_repo = Arc::new(ContentConfigRepository::new(&db));
    let game_repo = Arc::new(GameModelRepository::new(&db));

    let state = ContentState {
        config_repo,
        game_repo,
    };

    Router::new().route("/", get(get_content)).with_state(state)
}
