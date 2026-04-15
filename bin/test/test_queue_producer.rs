// src/bin/test_queue_producer.rs
// Test producer — pushes MigrationJob payloads to the queue.
// Run this in one terminal, and `moments_worker` in another.

use kult_browser_backend_rust::moments::{MigrationJob, MIGRATION_QUEUE};
use kult_browser_backend_rust::redis::{connect, ValkyQueue};
use std::time::Duration;

// ──────────────────────────────────────────────
// TWEAK THESE
// ──────────────────────────────────────────────

/// Delay between each push (ms)
const PUSH_INTERVAL_MS: u64 = 500;

/// Total number of jobs to push
const TOTAL_JOBS: u32 = 5;

// ──────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("=== Queue Producer Test ===");
    println!(
        "Push interval: {}ms | Total jobs: {}\n",
        PUSH_INTERVAL_MS, TOTAL_JOBS
    );

    let client = connect().expect("Failed to connect to Valkey");
    let queue = ValkyQueue::new(client, MIGRATION_QUEUE).await.expect("Failed to create queue connection");

    for i in 1..=TOTAL_JOBS {
        let job = MigrationJob {
            asset_url: format!(
                "https://kult-browser.sfo3.cdn.digitaloceanspaces.com/moments/test_{}.gif",
                i
            ),
            asset_id: format!("test-asset-{}", i),
            asset_type: "image/gif".to_string(),
            attempt: 1,
        };

        queue.push(&job).expect("Failed to push");
        let len = queue.len().unwrap_or(0);
        println!("[PRODUCER] Pushed job #{} | Queue length: {}", i, len);

        tokio::time::sleep(Duration::from_millis(PUSH_INTERVAL_MS)).await;
    }

    println!("\n[PRODUCER] Done — pushed {} jobs", TOTAL_JOBS);
    println!("Check the moments_worker terminal to see them being consumed.");
}
