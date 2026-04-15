use tokio::signal;
use tokio::sync::watch;

use kult_browser_backend_rust::config::CONFIG;
use kult_browser_backend_rust::moments::{MigrationWorker, MomentsRepository, MIGRATION_QUEUE};
use kult_browser_backend_rust::redis::{connect, ValkyQueue};

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting moments migration worker...");

    // Create a broadcast channel for graceful shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Setup signal handler
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

    // Connect to Valkey
    let client = connect().expect("Failed to connect to Valkey");
    let queue = ValkyQueue::new(client, MIGRATION_QUEUE).await.expect("Failed to create queue connection");

    // Connect to MongoDB
    let mongo_client = mongodb::Client::with_uri_str(&CONFIG.db.mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = mongo_client.database(&CONFIG.db.mongo_db_name);
    let repo = MomentsRepository::new(&db);

    tracing::info!("Connected to MongoDB ({})", CONFIG.db.mongo_db_name);

    // Create the ONE worker instance (singleton)
    let worker = MigrationWorker::new(queue, repo, shutdown_rx);

    // Run until shutdown signal
    worker.run().await;

    tracing::info!("Worker successfully shut down.");
}
