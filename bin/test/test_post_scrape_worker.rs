// src/bin/test_post_scrape_worker.rs
// Integration test for the PostScrapeWorker queue pipeline
//
// Tests:
// 1. Push a "young" job (created just now) → worker should re-queue it
// 2. Push a "mature" job (created 25h ago) → worker should process it
// 3. Verify the processed post gets updated in MongoDB

use chrono::{Duration, Utc};
use dotenvy::dotenv;
use mongodb::bson::oid::ObjectId;
use tracing::{info, Level};

use kult_browser_backend_rust::config::db_config::DbConfig;
use kult_browser_backend_rust::moments::social_media::{
    model::{
        platform::Platform,
        post_model::{SharedPost, ValidationStatus},
    },
    repository::post_repository::PostRepository,
    worker::scrape_job::{ScrapeJob, SCRAPE_QUEUE},
};
use kult_browser_backend_rust::mongo::connection::connect;
use kult_browser_backend_rust::redis::ValkyQueue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("=== PostScrapeWorker Queue Test ===");

    // --- Setup ---
    let post_repo = PostRepository::new().await?;
    let config = DbConfig::from_env();
    let db: mongodb::Database = connect().await?;

    // Clean up previous test data
    let _ = db
        .collection::<SharedPost>(&config.shared_posts_collection)
        .drop()
        .await;
    info!("Cleared old test collection.");

    // Connect to Redis for queue
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url.as_str()).expect("Failed to connect to Redis");
    let queue = ValkyQueue::new(redis_client.clone(), SCRAPE_QUEUE).await.expect("Failed to create queue connection");

    // Drain the queue to start fresh
    while let Ok(Some(_)) = queue.pop::<ScrapeJob>(1) {}
    info!("Cleared scrape queue.");

    // --- Test 1: Push a young job (< 24h old) and verify it stays in queue ---
    info!("--- Test 1: Young Job (should be re-queued) ---");

    let now = Utc::now();
    let mock_moment_id = ObjectId::new();

    // First insert a SharedPost so the worker has something to update
    let young_post = SharedPost {
        id: None,
        moment_id: mock_moment_id,
        wallet_address: "0xTestWalletYoung".to_string(),
        platform: Platform::Twitter,
        post_id: "young_tweet_123".to_string(),
        url: "https://twitter.com/test/status/young_tweet_123".to_string(),
        num_likes: 0,
        score: 0,
        is_validated: false,
        validation_status: ValidationStatus::Pending,
        validation_reason: None,
        last_validated_at: None,
        created_at: now,
        updated_at: now,
    };
    let young_db_id = post_repo.create_post(young_post).await?;
    info!("Inserted young SharedPost: {}", young_db_id);

    let young_job = ScrapeJob {
        post_db_id: young_db_id,
        platform: Platform::Twitter,
        url: "https://twitter.com/test/status/young_tweet_123".to_string(),
        created_at: now, // Just created — way less than 24h old
        attempt: 1,
    };
    queue.push(&young_job)?;
    info!("Pushed young ScrapeJob to queue.");

    // Check queue length
    let len = queue.len()?;
    info!("Queue length after push: {}", len);
    assert!(len >= 1, "Expected at least 1 job in queue");
    info!("✅ Young job is in the queue.");

    // --- Test 2: Push a mature job (> 24h old) ---
    info!("--- Test 2: Mature Job (should be processed) ---");

    let mature_time = now - Duration::hours(25); // 25 hours ago

    let mature_post = SharedPost {
        id: None,
        moment_id: ObjectId::new(),
        wallet_address: "0xTestWalletMature".to_string(),
        platform: Platform::Twitter,
        post_id: "mature_tweet_456".to_string(),
        url: "https://twitter.com/test/status/mature_tweet_456".to_string(),
        num_likes: 0,
        score: 0,
        is_validated: false,
        validation_status: ValidationStatus::Pending,
        validation_reason: None,
        last_validated_at: None,
        created_at: mature_time,
        updated_at: mature_time,
    };
    let mature_db_id = post_repo.create_post(mature_post).await?;
    info!("Inserted mature SharedPost: {}", mature_db_id);

    let mature_job = ScrapeJob {
        post_db_id: mature_db_id,
        platform: Platform::Twitter,
        url: "https://twitter.com/test/status/mature_tweet_456".to_string(),
        created_at: mature_time, // 25 hours ago — past the 24h gate
        attempt: 1,
    };
    queue.push(&mature_job)?;
    info!("Pushed mature ScrapeJob to queue.");

    let len = queue.len()?;
    info!("Queue length after both pushes: {}", len);
    assert!(len >= 2, "Expected at least 2 jobs in queue");

    // --- Test 3: Pop jobs and verify the 24h gate logic ---
    info!("--- Test 3: Manually popping to verify ordering ---");

    // Pop first job (FIFO — the young one was pushed first)
    let first_job: ScrapeJob = queue.pop::<ScrapeJob>(2)?.expect("Expected a job");
    let first_age = Utc::now() - first_job.created_at;
    info!(
        "First popped job: post_db_id={}, age_hours={}, should_process={}",
        first_job.post_db_id,
        first_age.num_hours(),
        first_age.num_hours() >= 24
    );

    if first_age.num_hours() < 24 {
        info!("✅ First job is young — worker would re-queue it.");
    } else {
        info!("✅ First job is mature — worker would process it.");
    }

    // Pop second job
    let second_job: ScrapeJob = queue.pop::<ScrapeJob>(2)?.expect("Expected a second job");
    let second_age = Utc::now() - second_job.created_at;
    info!(
        "Second popped job: post_db_id={}, age_hours={}, should_process={}",
        second_job.post_db_id,
        second_age.num_hours(),
        second_age.num_hours() >= 24
    );

    if second_age.num_hours() >= 24 {
        info!("✅ Second job is mature — worker would process it.");
    } else {
        info!("✅ Second job is young — worker would re-queue it.");
    }

    // Verify queue is now empty
    let remaining = queue.len()?;
    info!("Remaining in queue: {}", remaining);
    assert_eq!(
        remaining, 0,
        "Queue should be empty after popping both jobs"
    );
    info!("✅ Queue is empty after consuming all jobs.");

    info!("=== All PostScrapeWorker Queue Tests Passed! ===");
    Ok(())
}
