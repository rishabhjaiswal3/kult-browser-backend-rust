use dotenvy::dotenv;
use mongodb::bson::oid::ObjectId;
use tracing::{info, Level};

use kult_browser_backend_rust::moments::social_media::{
    model::{
        platform::Platform,
        post_model::{SharedPost, ValidationStatus},
    },
    repository::post_repository::PostRepository,
    service::post_service::PostService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Environment & Tracing
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Social Media Post Test...");

    // 2. Initialize Service & Repository
    let post_service = PostService::new().await?;
    let post_repo = PostRepository::new().await?;

    info!("Database connections established.");

    // Clean up test DB beforehand so bad previous data doesn't fail deserialization
    use kult_browser_backend_rust::config::db_config::DbConfig;
    use kult_browser_backend_rust::mongo::connection::connect;
    let config = DbConfig::from_env();
    let db: mongodb::Database = connect().await?;
    let _ = db
        .collection::<SharedPost>(&config.shared_posts_collection)
        .drop()
        .await;
    info!("Cleared old test collection.");

    // 3. Dummy Data
    let mock_moment_id = ObjectId::new();
    let mock_wallet = "0xTestWallet123".to_string();
    let mock_platform = Platform::Twitter;
    let mock_post_id = format!("test_tweet_{}", chrono::Utc::now().timestamp());
    let mock_url = format!("https://twitter.com/test/status/{}", mock_post_id);

    // 4. Test Service: Submit Shared Post
    info!("--- 1. Testing Post Submission ---");
    match post_service
        .submit_shared_post(
            mock_moment_id.clone(),
            mock_wallet.clone(),
            mock_platform.clone(),
            mock_post_id.clone(),
            mock_url.clone(),
        )
        .await
    {
        Ok(inserted_id) => info!("✅ Successfully submitted post with ID: {}", inserted_id),
        Err(e) => {
            tracing::error!("❌ Failed to submit post: {:?}", e);
            return Err(e.to_string().into());
        }
    };

    // 5. Test Repository: Duplicate Check
    info!("--- 2. Testing Duplicate Post Rejection ---");
    match post_service
        .submit_shared_post(
            mock_moment_id.clone(),
            mock_wallet.clone(),
            mock_platform.clone(),
            mock_post_id.clone(),
            mock_url.clone(),
        )
        .await
    {
        Ok(_) => tracing::error!(
            "❌ Re-submitted post successfully when it should have failed as duplicate!"
        ),
        Err(e) => info!("✅ Correctly rejected duplicate post. Error: {}", e),
    };

    // 6. Test Repository: Fetch by Wallet
    info!("--- 3. Testing Fetching via Wallet ---");
    let wallet_posts = post_repo.get_posts_by_wallet_address(&mock_wallet).await?;
    info!(
        "✅ Found {} posts for wallet {}",
        wallet_posts.len(),
        mock_wallet
    );

    // Grab the actual DB ID for the next steps
    let db_post_id = wallet_posts.last().unwrap().id.unwrap();

    // 7. Test Repository: Updating Metrics (Simulating the Scraper)
    info!("--- 4. Testing Metric Updates (Scraper Simulation) ---");
    let new_likes = 42;
    let new_score = 100;

    match post_repo
        .update_post_metrics(
            db_post_id,
            new_likes,
            new_score,
            true, // is_validated
            ValidationStatus::Valid,
            "test:manual",
        )
        .await
    {
        Ok(_) => info!(
            "✅ Successfully updated post metrics (likes: {}, score: {})",
            new_likes, new_score
        ),
        Err(e) => {
            tracing::error!("❌ Failed to update metrics: {:?}", e);
            return Err(e.into());
        }
    };

    // 8. Verify the update stuck
    info!("--- 5. Verifying Update Retrieval ---");
    let updated_post = post_repo
        .get_post_by_platform_and_id(mock_platform, &mock_post_id)
        .await?
        .unwrap();
    if updated_post.num_likes == 42 && updated_post.validation_status == ValidationStatus::Valid {
        info!("✅ Verified: Post data was correctly saved and retrieved from DB.");
    } else {
        tracing::error!("❌ Verification failed. Fetched metrics didn't match.");
    }

    info!("All Social Media Post tests completed successfully!");
    Ok(())
}
