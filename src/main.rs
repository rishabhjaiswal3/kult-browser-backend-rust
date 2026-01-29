use axum::{routing::get, Router};
use kult_browser_backend_rust::{
    content::{
        controller::content_controller::{get_content, ContentState},
        repository::ContentConfigRepository,
    },
    game::repository::GameModelRepository,
    mongo::connection,
};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");
    println!("Database connected: {}", db.name());

    let game_repo = Arc::new(GameModelRepository::new(&db));
    let config_repo = Arc::new(ContentConfigRepository::new(&db));

    let content_state = ContentState {
        config_repo: config_repo.clone(),
        game_repo: game_repo.clone(),
    };

    let app = Router::new()
        .route("/api/content", get(get_content))
        .with_state(content_state);

    println!("Server running on port 3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
