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

    // Start the server - exit if it fails
    if let Err(e) = server::run(db).await {
        tracing::error!(error = %e, "Server error, shutting down");
        std::process::exit(1);
    }
}
