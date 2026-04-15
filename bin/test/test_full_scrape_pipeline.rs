// src/bin/test_full_scrape_pipeline.rs
// End-to-end integration test for the full scrape pipeline
//
// Simulates:
// 1. Player submits a real Twitter URL as a SharedPost
// 2. PostScraperService scrapes the post via BrightData
// 3. Validates the post, extracts likes, calculates score
// 4. Updates the SharedPost in MongoDB with the scraped metrics
//
// ⚠️ This test calls BrightData's live API — it will consume credits.

use chrono::{Duration, Utc};
use dotenvy::dotenv;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use tracing::{info, Level};

use kult_browser_backend_rust::config::db_config::DbConfig;
use kult_browser_backend_rust::moments::social_media::{
    model::{
        platform::Platform,
        post_model::{SharedPost, ValidationStatus},
    },
    repository::post_repository::PostRepository,
    service::post_scraper_service::PostScraperService,
};
use kult_browser_backend_rust::mongo::connection::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("=== Full Scrape Pipeline Test ===");

    // --- Setup ---
    let config = DbConfig::from_env();
    let db: mongodb::Database = connect().await?;
    let post_repo = PostRepository::new(&db);
    let collection = db.collection::<SharedPost>(&config.shared_posts_collection);

    let test_wallet = format!("0xTestWalletFullPipeline{}", Utc::now().timestamp_millis());
    collection
        .delete_many(doc! { "wallet_address": &test_wallet })
        .await?;

    // --- Step 1: Insert a mock SharedPost with a REAL Twitter URL ---
    let real_twitter_url = "https://x.com/Ronnie_Ree/status/2020518068800160166";
    let now = Utc::now();
    let created_time = now - Duration::hours(25); // Simulate 25h ago so gate passes

    let test_post = SharedPost {
        id: None,
        moment_id: Bson::ObjectId(ObjectId::new()),
        wallet_address: test_wallet.clone(),
        platform: Platform::Twitter,
        post_id: "2020518068800160166".to_string(),
        url: real_twitter_url.to_string(),
        num_likes: 0,
        score: 0,
        is_validated: false,
        validation_status: ValidationStatus::Pending,
        validation_reason: None,
        last_validated_at: None,
        created_at: created_time,
        updated_at: created_time,
    };

    let post_db_id = post_repo.create_post(test_post).await?;
    info!("Inserted test SharedPost: {}", post_db_id);

    // --- Step 2: Call PostScraperService directly (bypassing queue for this test) ---
    let scraper_service = PostScraperService::new(post_repo.clone());

    info!("--- Calling BrightData to scrape: {} ---", real_twitter_url);

    let result = scraper_service
        .scrape_and_validate(post_db_id, &Platform::Twitter, real_twitter_url)
        .await
        .map_err(|err| format!("scrape_and_validate failed: {err}"))?;

    info!("Scrape succeeded");
    info!("   Likes:  {}", result.likes);
    info!("   Score:  {}", result.score);
    info!("   Valid:  {}", result.is_valid);
    info!("   Status: {:?}", result.status);

    if result.status != ValidationStatus::Invalid {
        return Err(format!(
            "expected Invalid status for non-Kult tweet, got {:?}",
            result.status
        )
        .into());
    }

    // --- Step 3: Verify the data was updated in MongoDB ---
    info!("--- Verifying MongoDB was updated ---");
    let post = collection
        .find_one(doc! { "_id": post_db_id })
        .await?
        .ok_or("No post found in MongoDB after scrape")?;

    if !post.is_validated {
        return Err("post was not marked as validated".into());
    }
    if post.validation_status != ValidationStatus::Invalid {
        return Err(format!(
            "expected stored validation_status=Invalid, got {:?}",
            post.validation_status
        )
        .into());
    }
    if post.validation_reason.is_none() {
        return Err("validation_reason was not stored".into());
    }

    info!("MongoDB post after scrape:");
    info!("   num_likes:         {}", post.num_likes);
    info!("   score:             {}", post.score);
    info!("   is_validated:      {}", post.is_validated);
    info!("   validation_status: {:?}", post.validation_status);
    info!("   validation_reason: {:?}", post.validation_reason);
    info!("   last_validated_at: {:?}", post.last_validated_at);

    collection
        .delete_many(doc! { "wallet_address": &test_wallet })
        .await?;

    info!("=== Full Pipeline Test Complete ===");
    Ok(())
}
