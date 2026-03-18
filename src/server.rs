use axum::{http::StatusCode, routing::get, Json, Router};
use mongodb::Database;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::CONFIG;
use crate::content;
use crate::game;
use crate::leaderboard;
use crate::moments;
use crate::moments::social_media;
use crate::moments::social_media::worker::scrape_job::SCRAPE_QUEUE;
use crate::moments::MIGRATION_QUEUE;
use crate::player;
use crate::redis::{connect as valkey_connect, ValkyQueue};

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Health check response", body = crate::openapi::HealthResponse)
    ),
    tag = "Health"
)]
pub(crate) async fn health_check() -> Json<serde_json::Value> {
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

use crate::referral;
use std::sync::Arc;

/// Build the application router with all routes
async fn build_router(db: Database) -> Router {
    let client = db.client().clone();

    // Connect to Valkey for queues
    let (migration_queue, scrape_queue, anti_fraud_service, ref_app_state, referral_router) =
        match valkey_connect() {
            Ok(valkey_client) => {
                tracing::info!("Connected to Valkey for queues");

                // Queue instances
                let m_queue = ValkyQueue::new(valkey_client.clone(), MIGRATION_QUEUE);
                let s_queue = ValkyQueue::new(valkey_client.clone(), SCRAPE_QUEUE);
                let c_queue = ValkyQueue::new(valkey_client.clone(), referral::CLICK_QUEUE);
                let v_queue = ValkyQueue::new(valkey_client.clone(), referral::VERIFY_QUEUE);

                // Referral Services
                let player_repo = Arc::new(player::PlayerRepository::new(&db));
                let referral_service = Arc::new(referral::service::ReferralService::new(
                    player_repo,
                    valkey_client.clone(),
                ));
                let click_analytics =
                    Arc::new(referral::analytics::ClickAnalyticsService::new(c_queue));
                let anti_fraud = Arc::new(referral::anti_fraud::AntiFraudService::new(
                    valkey_client,
                    v_queue,
                ));

                let ref_app_state = referral::redirect_route::RedirectAppState {
                    referral_service: referral_service.clone(),
                    click_analytics,
                };

                let ref_router = referral::route::router().with_state(referral_service);

                (
                    Some(m_queue),
                    Some(s_queue),
                    Some(anti_fraud),
                    Some(ref_app_state),
                    Some(ref_router),
                )
            }
            Err(e) => {
                tracing::warn!(error = %e, "Valkey not available — queues disabled");
                (None, None, None, None, None)
            }
        };

    // HTTP request/response tracing middleware
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // CORS Middleware
    let cors = if CONFIG.app.cors_origins.len() == 1 && CONFIG.app.cors_origins[0] == "*" {
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    } else {
        let origins: Vec<axum::http::HeaderValue> = CONFIG
            .app
            .cors_origins
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        tower_http::cors::CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(true)
    };

    let mut router = Router::new()
        .route("/api/health", get(health_check))
        .nest("/api/content", content::routes(db.clone()))
        .nest("/api/games", game::routes(db.clone()))
        .nest(
            "/api/leaderboard",
            leaderboard::routes(db.clone(), client.clone()),
        )
        .nest(
            "/api/player",
            player::routes(db.clone(), client, anti_fraud_service),
        )
        .nest("/api/moments", moments::routes(db.clone(), migration_queue))
        .nest(
            "/api/moments/social-media",
            social_media::route::routes(scrape_queue).await,
        )
        .nest(
            "/api/upload",
            axum::Router::new().route(
                "/presign",
                axum::routing::post(crate::upload::controller::generate_presigned_url),
            ),
        );

    // Only mount referral routes if Redis is available
    if let (Some(state), Some(referral_routes)) = (ref_app_state, referral_router) {
        let redirect_routes = referral::redirect_route::router().with_state(state);
        router = router
            .nest("/api/referral", referral_routes)
            .nest("/r", redirect_routes);
    }

    router
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", crate::openapi::ApiDoc::openapi()),
        )
        .fallback(fallback)
        .layer(trace_layer)
        .layer(cors)
}

/// Start the HTTP server
pub async fn run(
    db: Database,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), std::io::Error> {
    let app = build_router(db).await;

    let host = &CONFIG.app.host;
    let port = CONFIG.app.port;
    let addr = format!("{}:{}", host, port);

    tracing::info!(host = %host, port = %port, "Server starting");

    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.changed().await;
            tracing::info!("HTTP Server received shutdown signal.");
        })
        .await
}
