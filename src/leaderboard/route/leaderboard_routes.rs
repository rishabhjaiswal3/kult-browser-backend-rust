use crate::game::repository::GameModelRepository;
use axum::{
    routing::{get, post},
    Router,
};
use mongodb::{Client, Database};

use crate::leaderboard::controller::game_leaderboard_controller::get_game_leaderboard;
use crate::leaderboard::controller::leaderboard_controller::{
    get_global_leaderboard, refresh_global_leaderboard,
};
use crate::leaderboard::controller::LeaderboardState;
use crate::leaderboard::repository::{
    GameLeaderboardConfigRepository, GlobalLeaderboardRepository,
};
use crate::leaderboard::service::{GameLeaderboardService, GlobalLeaderboardService};

/// Build routes for the leaderboard module.
///
/// Routes:
/// - GET /global - Get global leaderboard (paginated)
/// - GET /game/:identification - Get game-specific leaderboard
/// - POST /refresh - Refresh global leaderboard (admin)
pub fn routes(db: Database, client: Client) -> Router {
    let config_repo = GameLeaderboardConfigRepository::new(&db);
    let global_repo = GlobalLeaderboardRepository::new(&db);
    let game_repo = GameModelRepository::new(&db);
    let game_service = GameLeaderboardService::new(config_repo.clone(), client);
    let global_service =
        GlobalLeaderboardService::new(config_repo, global_repo, game_service.clone(), game_repo);

    let state = LeaderboardState {
        game_service,
        global_service,
    };

    Router::new()
        .route("/global", get(get_global_leaderboard))
        .route("/game/:identification", get(get_game_leaderboard))
        .route("/refresh", post(refresh_global_leaderboard))
        .with_state(state)
}
