use axum::{http::StatusCode, routing::get, Json, Router};
use mongodb::Database;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::config::CONFIG;
use crate::content;
use crate::game;
use crate::leaderboard;
use crate::moments;
use crate::player;

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "ts": chrono::Utc::now().to_rfc3339()
    }))
}

/// Fallback handler for unmatched routes - returns structured 404
async fn fallback() -> (StatusCode, Json<serde_json::Value>) {
    tracing::warn!("Route not found");
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "ok": false,
            "message": "Route not found"
        })),
    )
}

/// Build the application router with all routes
fn build_router(db: Database) -> Router {
    let client = db.client().clone();

    // HTTP request/response tracing middleware
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    Router::new()
        .route("/api/health", get(health_check))
        .nest("/api/content", content::routes(db.clone()))
        .nest("/api/games", game::routes(db.clone()))
        .nest(
            "/api/leaderboard",
            leaderboard::routes(db.clone(), client.clone()),
        )
        .nest("/api/player", player::routes(db.clone(), client))
        .nest("/api/moments", moments::routes(db.clone(), None))
        .fallback(fallback)
        .layer(trace_layer)
}

/// Start the HTTP server
pub async fn run(db: Database) -> Result<(), std::io::Error> {
    let app = build_router(db);

    let host = &CONFIG.app.host;
    let port = CONFIG.app.port;
    let addr = format!("{}:{}", host, port);

    tracing::info!(host = %host, port = %port, "Server starting");

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await
}
