/// Final retry for Twitter and TikTok — using URLs that returned valid data earlier.
/// Run: cargo run --bin scrape_final
use dotenvy::dotenv;
use kult_browser_backend_rust::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;
use std::io::Write;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let scraper = BrightDataPostScraper::new();
    let data_dir = std::path::PathBuf::from("data");

    // Twitter — PlayStation post (confirmed working)
    tracing::info!("=== Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/PlayStation/status/2024484765882339769".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("twitter.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Twitter OK");
        }
        Err(e) => tracing::error!(error = %e, "Twitter FAIL"),
    }

    // TikTok — Highland Bros dance video (confirmed working)
    tracing::info!("=== TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("tiktok.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "TikTok OK");
        }
        Err(e) => tracing::error!(error = %e, "TikTok FAIL"),
    }

    tracing::info!("=== Done! ===");
    Ok(())
}
