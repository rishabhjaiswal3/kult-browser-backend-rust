use kult_browser_backend_rust::moments::{MigrationWorker, MomentsRepository, MIGRATION_QUEUE};
use kult_browser_backend_rust::redis::{connect as valkey_connect, ValkyQueue};
use kult_browser_backend_rust::{logging, mongo, server};

#[tokio::main]
async fn main() {
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

    // Spawn migration worker as a background task
    match valkey_connect() {
        Ok(valkey_client) => {
            let queue = ValkyQueue::new(valkey_client, MIGRATION_QUEUE);
            let repo = MomentsRepository::new(&db);
            let worker = MigrationWorker::new(queue, repo);

            tokio::spawn(async move {
                worker.run().await;
            });

            tracing::info!("Migration worker spawned as background task");
        }
        Err(e) => {
            tracing::warn!(error = %e, "Valkey not available — migration worker disabled");
        }
    }

    // Start the server - exit if it fails
    if let Err(e) = server::run(db).await {
        tracing::error!(error = %e, "Server error, shutting down");
        std::process::exit(1);
    }
}
