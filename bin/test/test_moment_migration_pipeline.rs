// src/bin/test_moment_migration_pipeline.rs
// Live end-to-end test for the moments 0G migration path.
//
// Flow:
// 1. Upload a small text file to DigitalOcean Spaces via presigned URL.
// 2. Create a moment through MomentsService with assetUrl + assetMetadata.fileType.
// 3. Verify a migration job is pushed to a dedicated Redis queue.
// 4. Run MigrationWorker on that dedicated queue.
// 5. Assert assetZgHash is written back to the moment document.

use std::time::Duration;

use dotenvy::dotenv;
use mongodb::Database;
use reqwest::Client as HttpClient;
use serde_json::json;
use tokio::sync::watch;
use tokio::time::{sleep, Instant};

use kult_browser_backend_rust::config::CONFIG;
use kult_browser_backend_rust::external::digital_ocean::spaces::SpacesService;
use kult_browser_backend_rust::moments::dto::CreateMomentRequest;
use kult_browser_backend_rust::moments::repository::MomentsRepository;
use kult_browser_backend_rust::moments::service::MomentsService;
use kult_browser_backend_rust::moments::worker::MigrationWorker;
use kult_browser_backend_rust::mongo::connection::connect as connect_mongo;
use kult_browser_backend_rust::redis::{connect as connect_valkey, ValkyQueue};

fn build_public_url(filename: &str) -> String {
    let endpoint_clean = CONFIG
        .do_spaces
        .endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let upload_path = CONFIG.do_spaces.effective_upload_path();
    let key = format!("{}/{}", upload_path, filename);
    format!(
        "https://{}.{}/{}",
        CONFIG.do_spaces.bucket, endpoint_clean, key
    )
}

fn clear_queue_keys(client: &redis::Client, queue_name: &str) -> Result<(), String> {
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("Valkey connection error: {}", e))?;

    let processing = format!("{queue_name}:processing");
    let dead_letter = format!("{queue_name}:dead_letter");
    let dead_letter_processing = format!("{dead_letter}:processing");

    redis::cmd("DEL")
        .arg(queue_name)
        .arg(processing)
        .arg(dead_letter)
        .arg(dead_letter_processing)
        .query::<i64>(&mut conn)
        .map_err(|e| format!("DEL error: {}", e))?;

    Ok(())
}

fn queue_len(client: &redis::Client, queue_name: &str) -> Result<u64, String> {
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("Valkey connection error: {}", e))?;

    redis::cmd("LLEN")
        .arg(queue_name)
        .query::<u64>(&mut conn)
        .map_err(|e| format!("LLEN error: {}", e))
}

async fn upload_test_file(
    spaces_service: &SpacesService,
    filename: &str,
    content_type: &str,
    body: Vec<u8>,
) -> Result<String, String> {
    let presigned = spaces_service
        .generate_presigned_upload_url(filename, content_type)
        .await?;

    let mut request = HttpClient::new()
        .put(presigned.uri().to_string())
        .body(body);

    for (name, value) in presigned.headers() {
        request = request.header(name, value.to_string());
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Spaces upload request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Spaces upload failed: HTTP {} {}", status, body));
    }

    let public_url = build_public_url(filename);

    for _ in 0..15 {
        if spaces_service.check_file_exists(&public_url).await {
            return Ok(public_url);
        }
        sleep(Duration::from_secs(2)).await;
    }

    Err(format!(
        "Uploaded object never became visible in Spaces: {}",
        public_url
    ))
}

async fn wait_for_zg_hash(
    repo: &MomentsRepository,
    valkey_client: &redis::Client,
    queue_name: &str,
    moment_id: &str,
    timeout: Duration,
) -> Result<String, String> {
    let started = Instant::now();
    let dead_letter_queue = format!("{queue_name}:dead_letter");

    loop {
        if queue_len(valkey_client, &dead_letter_queue)? > 0 {
            return Err(format!(
                "Migration job moved to dead-letter queue: {}",
                dead_letter_queue
            ));
        }

        let moment = repo
            .find_by_moment_id(moment_id)
            .await?
            .ok_or_else(|| format!("Moment not found while polling: {}", moment_id))?;

        if let Some(hash) = moment.asset_zg_hash {
            return Ok(hash);
        }

        if started.elapsed() >= timeout {
            return Err(format!(
                "Timed out waiting for assetZgHash on moment {}",
                moment_id
            ));
        }

        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Moment 0G Migration Pipeline Test ===");

    let db: Database = connect_mongo().await.map_err(|e| e.to_string())?;
    let repo = MomentsRepository::new(&db);
    let spaces_service = SpacesService::new();
    let valkey_client = connect_valkey().await.map_err(|e| e.to_string())?;

    let test_queue_name = format!(
        "{}:moments:zero_g:migration:test:{}:queue",
        CONFIG.valkey.key_prefix,
        nanoid::nanoid!(8)
    );
    clear_queue_keys(&valkey_client, &test_queue_name)?;
    let queue = ValkyQueue::new(valkey_client.clone(), &test_queue_name)
        .await
        .expect("Failed to create queue connection");

    let filename = format!("0g-moment-test-{}.txt", nanoid::nanoid!(8));
    let file_body = format!(
        "Moment migration test\ncreated_at={}\n",
        chrono::Utc::now().to_rfc3339()
    )
    .into_bytes();

    println!("Uploading fixture to Spaces...");
    let public_url = upload_test_file(&spaces_service, &filename, "text/plain", file_body).await?;
    println!("Spaces upload verified: {}", public_url);

    let wallet = "0x0gtest000000000000000000000000000000000001";
    let service = MomentsService::with_queue(repo.clone(), queue.clone(), spaces_service.clone());

    let create_response = service
        .create_moment(
            wallet,
            CreateMomentRequest {
                asset_url: Some(public_url.clone()),
                asset_metadata: Some(json!({
                    "fileType": "text/plain",
                    "source": "test_moment_migration_pipeline"
                })),
                title: "0G Migration Test".to_string(),
                description: Some("Live end-to-end migration test".to_string()),
                tags: vec!["test".to_string(), "0g".to_string()],
                social_media_links: None,
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    println!("Moment created: {}", create_response.moment_id);

    let queued_jobs = queue.len_async().await?;
    if queued_jobs == 0 {
        let _ = repo.delete(&create_response.moment_id).await;
        let _ = clear_queue_keys(&valkey_client, &test_queue_name);
        return Err("No migration job was queued".to_string());
    }
    println!("Queued jobs: {}", queued_jobs);

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let worker = MigrationWorker::new(queue.clone(), repo.clone(), shutdown_rx);
    let worker_handle = tokio::spawn(async move {
        worker.run().await;
    });

    let zg_hash = match wait_for_zg_hash(
        &repo,
        &valkey_client,
        &test_queue_name,
        &create_response.moment_id,
        Duration::from_secs(240),
    )
    .await
    {
        Ok(hash) => hash,
        Err(err) => {
            let _ = shutdown_tx.send(true);
            worker_handle.abort();
            let _ = repo.delete(&create_response.moment_id).await;
            let _ = clear_queue_keys(&valkey_client, &test_queue_name);
            return Err(err);
        }
    };

    println!("assetZgHash updated: {}", zg_hash);

    let _ = shutdown_tx.send(true);
    let _ = tokio::time::timeout(Duration::from_secs(10), worker_handle).await;

    repo.delete(&create_response.moment_id).await?;
    clear_queue_keys(&valkey_client, &test_queue_name)?;

    println!("Moment 0G migration pipeline verified successfully.");
    Ok(())
}
