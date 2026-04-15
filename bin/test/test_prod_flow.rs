/// Full production flow test:
///
/// 1. Insert a SharedPost into MongoDB (status: Pending)
/// 2. Push a ScrapeJob to the Redis queue
/// 3. Start the PostScrapeWorker (with SCRAPE_MIN_AGE_SECS=5)
/// 4. Wait for the worker to process the job
/// 5. Read the SharedPost back from MongoDB and verify it was updated
///
/// Run: cargo run --bin test_prod_flow
use chrono::Utc;
use dotenvy::dotenv;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use std::time::Duration;
use tracing::Level;

use kult_browser_backend_rust::config::db_config::DbConfig;
use kult_browser_backend_rust::moments::social_media::model::platform::Platform;
use kult_browser_backend_rust::moments::social_media::model::post_model::{
    SharedPost, ValidationStatus,
};
use kult_browser_backend_rust::moments::social_media::repository::post_repository::PostRepository;
use kult_browser_backend_rust::moments::social_media::worker::post_scrape_worker::PostScrapeWorker;
use kult_browser_backend_rust::moments::social_media::worker::scrape_job::ScrapeJob;
use kult_browser_backend_rust::mongo::connection::connect;
use kult_browser_backend_rust::redis::connect as connect_valkey;
use kult_browser_backend_rust::redis::ValkyQueue;

fn main() {
    dotenv().ok();

    let _ = rustls::crypto::ring::default_provider().install_default();

    std::env::set_var("SCRAPE_MIN_AGE_SECS", "5");
    std::env::set_var("SCRAPE_REQUEUE_SLEEP_SECS", "1");
    std::env::set_var("SCRAPE_VALIDATION_TERMS", "game");

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║   FULL PRODUCTION FLOW TEST (age gate = 5 sec)          ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        // ── Setup ──
        let db = connect().await.expect("Failed to connect to MongoDB");
        let repo = PostRepository::new(&db);
        let db_config = DbConfig::from_env();
        let collection = db.collection::<SharedPost>(&db_config.shared_posts_collection);

        println!("Connecting to Valkey...");
        let valkey_url = std::env::var("VALKEY_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let valkey_client = connect_valkey()
            .await
            .expect("Failed to connect to Valkey");
        let queue_name = format!(
            "{}:moments:bright_data:post_scrape:test_prod_flow:{}:queue",
            kult_browser_backend_rust::config::CONFIG.valkey.key_prefix,
            nanoid::nanoid!(8)
        );
        let queue = ValkyQueue::new(valkey_client.clone(), &queue_name).await.expect("Failed to create queue connection");

        let processing_queue = format!("{}:processing", queue_name);
        let dead_letter_queue = format!("{}:dead_letter", queue_name);
        let dead_letter_processing = format!("{}:processing", dead_letter_queue);
        let mut cleanup_conn = valkey_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get async Valkey connection");
        redis::cmd("DEL")
            .arg(&queue_name)
            .arg(&processing_queue)
            .arg(&dead_letter_queue)
            .arg(&dead_letter_processing)
            .query_async::<i64>(&mut cleanup_conn)
            .await
            .expect("Failed to clear test queue keys");
        println!("Valkey connected. Queue key: {}", queue_name);

        let test_wallet = format!("0xTEST_WALLET_PROD_FLOW_{}", nanoid::nanoid!(8));
        collection
            .delete_many(doc! { "wallet_address": &test_wallet })
            .await
            .expect("Failed to clear old test wallet posts");

        // ── Test posts: one per platform ──
        // Using the same URLs from our previous tests
        let test_cases: Vec<(&str, Platform, &str)> = vec![
            ("Twitter",   Platform::Twitter,   "https://x.com/Ronnie_Ree/status/2020518068800160166"),
            ("Instagram", Platform::Instagram, "https://www.instagram.com/reel/CoZBDp2hu0H/"),
            ("TikTok",    Platform::TikTok,    "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255"),
            ("Facebook",  Platform::Facebook,  "https://www.facebook.com/kotaku/posts/1126531136015163/"),
            ("Reddit",    Platform::Reddit,    "https://www.reddit.com/r/gaming/comments/1rdppqz/1_million_in_debt_devs_on_handheld_tony_hawks_pro/"),
            ("LinkedIn",  Platform::LinkedIn,  "https://www.linkedin.com/posts/poki_gamedev-webgaming-monetisation-activity-7414576322225778688-o4T2"),
            ("Pinterest", Platform::Pinterest, "https://www.pinterest.com/pin/1110418851898928764/"),
        ];

        // ═══════════════════════════════════════
        // STEP 1: Insert SharedPosts into MongoDB
        // ═══════════════════════════════════════
        println!("━━━ STEP 1: Inserting SharedPosts into MongoDB ━━━\n");

        let now = Utc::now();
        let moment_id = Bson::ObjectId(ObjectId::new()); // Fake moment ID for testing
        let mut post_ids: Vec<(String, ObjectId, Platform, String)> = Vec::new();

        for (name, platform, url) in &test_cases {
            let post = SharedPost {
                id: None,
                moment_id: moment_id.clone(),
                wallet_address: test_wallet.clone(),
                platform: platform.clone(),
                post_id: format!("test_prod_{}", name.to_lowercase()),
                url: url.to_string(),
                num_likes: 0,
                score: 0,
                is_validated: false,
                validation_status: ValidationStatus::Pending,
                validation_reason: None,
                last_validated_at: None,
                created_at: now,
                updated_at: now,
            };

            match repo.create_post(post).await {
                Ok(db_id) => {
                    println!("  ✅ {} — inserted _id: {}", name, db_id);
                    post_ids.push((name.to_string(), db_id, platform.clone(), url.to_string()));
                }
                Err(e) => {
                    println!("  ❌ {} — insert failed: {}", name, e);
                }
            }
        }

        // ═══════════════════════════════════════
        // STEP 2: Push ScrapeJobs to Redis queue
        // ═══════════════════════════════════════
        println!("\n━━━ STEP 2: Pushing ScrapeJobs to Redis queue ━━━\n");

        for (name, db_id, platform, url) in &post_ids {
            let job = ScrapeJob {
                post_db_id: *db_id,
                platform: platform.clone(),
                url: url.clone(),
                created_at: now,
                attempt: 0,
            };

            match queue.push(&job) {
                Ok(_) => println!("  ✅ {} — pushed to queue (db_id: {})", name, db_id),
                Err(e) => println!("  ❌ {} — push failed: {}", name, e),
            }
        }

        let queue_len = queue.len().unwrap_or(0);
        println!("\n  Queue length: {}", queue_len);

        // ═══════════════════════════════════════
        // STEP 3: Start the worker (processes all jobs)
        // ═══════════════════════════════════════
        println!("\n━━━ STEP 3: Starting PostScrapeWorker (SCRAPE_MIN_AGE_SECS=5) ━━━\n");
        println!("  Worker will process {} jobs sequentially...\n", post_ids.len());
        println!("  Queue key: {}\n", queue_name);

        // Create a new queue + worker for the consumer
        let consumer_client = redis::Client::open(valkey_url.as_str())
            .expect("Failed to create Redis client for consumer");
        let consumer_queue = ValkyQueue::new(consumer_client, &queue_name).await.expect("Failed to create queue connection");
        
        let (shutdown_tx, rx) = tokio::sync::watch::channel(false);
        let worker = PostScrapeWorker::new(consumer_queue, repo.clone(), rx);

        // Run worker in background — it will process all jobs then block waiting
        let total_jobs = post_ids.len();
        let worker_handle = tokio::spawn(async move {
            worker.run().await;
        });

        // Wait for all jobs to be processed (check queue length)
        let mut wait_count = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            wait_count += 1;

            let remaining = queue.len_async().await.unwrap_or(0);
            println!("  ⏳ Waiting... ({}/{}s) — queue remaining: {}", wait_count * 5, 300, remaining);

            let all_posts = repo
                .get_posts_by_wallet_address(&test_wallet)
                .await
                .unwrap_or_default();
            let validated_now = all_posts.iter().filter(|p| p.is_validated).count();

            if remaining == 0 && validated_now == total_jobs {
                // Give the worker a few more seconds to finish the last job
                break;
            }

            if wait_count > 60 {
                // 5 min timeout
                println!("  ⚠️  Timeout — some jobs may not have completed");
                break;
            }
        }

        // ═══════════════════════════════════════
        // STEP 4: Verify MongoDB documents
        // ═══════════════════════════════════════
        println!("\n━━━ STEP 4: Verifying MongoDB documents ━━━\n");

        let all_posts = repo
            .get_posts_by_wallet_address(&test_wallet)
            .await
            .unwrap_or_default();

        println!("  Found {} posts for test wallet\n", all_posts.len());

        for post in &all_posts {
            let status_emoji = match post.validation_status {
                ValidationStatus::Valid => "✅",
                ValidationStatus::Invalid => "⚠️",
                ValidationStatus::Pending => "⏳",
            };

            println!("  ── {} {:?} ──", status_emoji, post.platform);
            println!("    _id:               {:?}", post.id);
            println!("    url:               {}", post.url);
            println!("    num_likes:         {}", post.num_likes);
            println!("    score:             {}", post.score);
            println!("    is_validated:      {}", post.is_validated);
            println!("    validation_status: {:?}", post.validation_status);
            println!("    validation_reason: {:?}", post.validation_reason);
            println!("    last_validated_at: {:?}", post.last_validated_at);
            println!();
        }

        // Summary
        let validated = all_posts.iter().filter(|p| p.is_validated).count();
        let still_pending = all_posts.iter().filter(|p| matches!(p.validation_status, ValidationStatus::Pending)).count();

        println!("  ── Summary ──");
        println!("  Total posts:    {}", all_posts.len());
        println!("  Validated:      {}", validated);
        println!("  Still pending:  {}", still_pending);

        let valid_count = all_posts
            .iter()
            .filter(|p| matches!(p.validation_status, ValidationStatus::Valid))
            .count();
        println!("  Valid:          {}", valid_count);

        if still_pending == 0 && validated == total_jobs && valid_count > 0 {
            println!("\n  🎉 ALL POSTS PROCESSED THROUGH FULL PIPELINE!");
        } else {
            println!("\n  ⚠️  Some posts may not have been processed yet");
            std::process::exit(1);
        }

        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║   Check MongoDB 'shared_posts' collection to see docs  ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        let _ = shutdown_tx.send(true);
        let _ = tokio::time::timeout(Duration::from_secs(10), worker_handle).await;

        collection
            .delete_many(doc! { "wallet_address": &test_wallet })
            .await
            .expect("Failed to clean test wallet posts");

        let mut cleanup_conn = valkey_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get async Valkey connection for cleanup");
        redis::cmd("DEL")
            .arg(&queue_name)
            .arg(&processing_queue)
            .arg(&dead_letter_queue)
            .arg(&dead_letter_processing)
            .query_async::<i64>(&mut cleanup_conn)
            .await
            .expect("Failed to clean queue keys");
    });
}
