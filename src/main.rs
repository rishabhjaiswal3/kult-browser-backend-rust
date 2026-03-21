use kult_browser_backend_rust::moments::social_media::repository::post_repository::PostRepository;
use kult_browser_backend_rust::moments::social_media::worker::{PostScrapeWorker, SCRAPE_QUEUE};
use kult_browser_backend_rust::moments::{MigrationWorker, MomentsRepository, MIGRATION_QUEUE};
use kult_browser_backend_rust::player::repository::PlayerRepository;
use kult_browser_backend_rust::redis::{connect as valkey_connect, ValkyQueue};
use kult_browser_backend_rust::referral::service::ReferralService;
use kult_browser_backend_rust::referral::worker::EvaluationWorker;
use kult_browser_backend_rust::referral::VERIFY_QUEUE;
use kult_browser_backend_rust::{logging, mongo, server};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    // Redis TLS via rustls requires an installed process-wide crypto provider.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize logging FIRST
    logging::init();

    // Connect to MongoDB - exit if it fails
    let db = match mongo::connect().await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!(error = %e, "Failed to connect to MongoDB, shutting down");
            std::process::exit(1);
        }
    };

    // Create a broadcast channel for graceful shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Setup signal handler for graceful shutdown (Ctrl-C / SIGTERM)
    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        tracing::info!("Shutdown signal received. Initiating graceful shutdown...");
        let _ = shutdown_tx.send(true);
    });

    let mut worker_handles = Vec::new();

    // Spawn background workers
    match valkey_connect().await {
        Ok(valkey_client) => {
            // Migration worker
            let migration_queue = ValkyQueue::new(valkey_client.clone(), MIGRATION_QUEUE);
            let repo = MomentsRepository::new(&db);
            let worker = MigrationWorker::new(migration_queue, repo, shutdown_rx.clone());

            let handle = tokio::spawn(async move {
                worker.run().await;
            });
            worker_handles.push(handle);
            tracing::info!("Migration worker spawned as background task");

            // Post scrape worker
            let scrape_queue = ValkyQueue::new(valkey_client.clone(), SCRAPE_QUEUE);
            match PostRepository::new().await {
                Ok(post_repo) => {
                    let scrape_worker =
                        PostScrapeWorker::new(scrape_queue, post_repo, shutdown_rx.clone());
                    let handle = tokio::spawn(async move {
                        scrape_worker.run().await;
                    });
                    worker_handles.push(handle);
                    tracing::info!("Post scrape worker spawned as background task");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create PostRepository — scrape worker disabled");
                }
            }

            // Referral evaluation worker
            let verify_queue = ValkyQueue::new(valkey_client.clone(), VERIFY_QUEUE);
            let player_repo = Arc::new(PlayerRepository::new(&db));
            let referral_service = Arc::new(ReferralService::new(player_repo, Some(valkey_client)));
            let eval_worker = EvaluationWorker::new(verify_queue, referral_service, db.clone());
            let eval_rx = shutdown_rx.clone();
            let handle = tokio::spawn(async move {
                eval_worker.run(eval_rx).await;
            });
            worker_handles.push(handle);
            tracing::info!("Referral evaluation worker spawned as background task");
        }
        Err(e) => {
            tracing::warn!(error = %e, "Valkey not available — background workers disabled");
        }
    }

    // Start the server (blocks until shutdown signal is received or crashes)
    if let Err(e) = server::run(db, shutdown_rx.clone()).await {
        tracing::error!(error = %e, "Server error, shutting down");
    }

    // After server shuts down from signal, wait for background workers to finish in-flight jobs
    tracing::info!("Waiting for background workers to complete active jobs...");
    for handle in worker_handles {
        let _ = handle.await;
    }

    tracing::info!("All processes successfully shut down. Goodbye!");
}
