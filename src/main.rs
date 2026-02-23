use kult_browser_backend_rust::moments::social_media::repository::post_repository::PostRepository;
use kult_browser_backend_rust::moments::social_media::worker::{PostScrapeWorker, SCRAPE_QUEUE};
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

    // Spawn background workers
    match valkey_connect() {
        Ok(valkey_client) => {
            // Migration worker
            let migration_queue = ValkyQueue::new(valkey_client.clone(), MIGRATION_QUEUE);
            let repo = MomentsRepository::new(&db);
            let worker = MigrationWorker::new(migration_queue, repo);

            tokio::spawn(async move {
                worker.run().await;
            });
            tracing::info!("Migration worker spawned as background task");

            // Post scrape worker
            let scrape_queue = ValkyQueue::new(valkey_client, SCRAPE_QUEUE);
            match PostRepository::new().await {
                Ok(post_repo) => {
                    let scrape_worker = PostScrapeWorker::new(scrape_queue, post_repo);
                    tokio::spawn(async move {
                        scrape_worker.run().await;
                    });
                    tracing::info!("Post scrape worker spawned as background task");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to create PostRepository — scrape worker disabled");
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Valkey not available — background workers disabled");
        }
    }

    // Start the server - exit if it fails
    if let Err(e) = server::run(db).await {
        tracing::error!(error = %e, "Server error, shutting down");
        std::process::exit(1);
    }
}
