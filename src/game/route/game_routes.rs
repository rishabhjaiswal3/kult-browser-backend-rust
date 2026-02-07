use axum::{routing::get, Router};
use mongodb::Database;

use crate::game::controller::{get_all_categories, get_all_games, get_game_by_id, GameState};
use crate::game::repository::GameModelRepository;
use crate::game::service::GameService;

/// Build routes for the game module.
///
/// Routes:
/// - GET /all - Get all games with pagination
/// - GET /all-categories - Get all unique categories  
/// - GET /:identification - Get a single game by ID
pub fn routes(db: Database) -> Router {
    let repo = GameModelRepository::new(&db);
    let service = GameService::new(repo);
    let state = GameState {
        game_service: service,
    };

    Router::new()
        .route("/all", get(get_all_games))
        .route("/all-categories", get(get_all_categories))
        .route("/:identification", get(get_game_by_id))
        .with_state(state)
}
