use axum::{routing::get, Router};
use mongodb::Database;

use crate::moments::creators::controller::{get_leaderboard, get_me, CreatorsState};
use crate::moments::creators::repository::CreatorsRepository;
use crate::moments::creators::service::CreatorsService;
use crate::player::repository::PlayerRepository;

pub fn routes(db: Database) -> Router {
    let creators_repository = CreatorsRepository::new(&db);
    let player_repository = PlayerRepository::new(&db);
    let service = CreatorsService::new(creators_repository, player_repository);
    let state = CreatorsState { service };

    Router::new()
        .route("/creators/me", get(get_me))
        .route("/creators/leaderboard", get(get_leaderboard))
        .with_state(state)
}
