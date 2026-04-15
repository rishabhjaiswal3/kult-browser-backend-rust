mod game_leaderboard_config_routes;
mod marketplace_admin_routes;

use axum::Router;
use mongodb::Database;

pub fn routes(db: Database) -> Router {
    let leaderboard_routes = game_leaderboard_config_routes::routes(db.clone());
    let marketplace_routes = marketplace_admin_routes::routes(db);

    leaderboard_routes.merge(marketplace_routes)
}
