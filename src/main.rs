use axum::{
    routing::{get, patch, post},
    Router,
};
use kult_browser_backend_rust::{
    content::{
        controller::content_controller::{get_content, ContentState},
        repository::ContentConfigRepository,
    },
    game::repository::GameModelRepository,
    leaderboard::{
        controller::{
            game_leaderboard_controller::get_game_leaderboard,
            leaderboard_controller::{get_global_leaderboard, refresh_global_leaderboard},
            LeaderboardState,
        },
        repository::{GameLeaderboardConfigRepository, GlobalLeaderboardRepository},
        service::{GameLeaderboardService, GlobalLeaderboardService},
    },
    mongo::connection,
    player::{
        controller::{get_profile, login, update_name, PlayerState},
        PlayerRepository, PlayerService,
    },
};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");
    println!("Database connected: {}", db.name());

    let client = db.client(); // Cloneable reference to client

    let game_repo = Arc::new(GameModelRepository::new(&db));
    let content_config_repo = Arc::new(ContentConfigRepository::new(&db));

    // Leaderboard Initialization
    let lb_config_repo = GameLeaderboardConfigRepository::new(&db);
    let lb_global_repo = GlobalLeaderboardRepository::new(&db);

    // Services (Owned)
    let game_lb_service = GameLeaderboardService::new(lb_config_repo.clone(), client.clone());
    let global_lb_service = GlobalLeaderboardService::new(
        lb_config_repo,
        lb_global_repo.clone(),
        game_lb_service.clone(),
    );

    // Player Initialization
    let player_repo = PlayerRepository::new(&db);
    let player_service = PlayerService::new(player_repo, lb_global_repo, game_lb_service.clone());

    let content_state = ContentState {
        config_repo: content_config_repo.clone(),
        game_repo: game_repo.clone(),
    };

    let leaderboard_state = LeaderboardState {
        game_service: game_lb_service,
        global_service: global_lb_service,
    };

    let player_state = PlayerState { player_service };

    let app = Router::new()
        .route("/api/content", get(get_content))
        .with_state(content_state)
        // Leaderboard Routes
        .nest(
            "/api/leaderboards",
            Router::new()
                .route("/game/:identification", get(get_game_leaderboard))
                .route("/global", get(get_global_leaderboard))
                .route("/global/refresh", post(refresh_global_leaderboard))
                .with_state(leaderboard_state),
        )
        // Player Routes
        .nest(
            "/api/player",
            Router::new()
                .route("/login", post(login))
                .route("/profile", get(get_profile))
                .route("/name", patch(update_name))
                .with_state(player_state),
        );

    println!("Server running on port 3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
