/// Retry scraping for TikTok, Reddit, and LinkedIn which failed in the first run.
/// Uses different TikTok URL (the previous one may have been removed).
///
/// Run: cargo run --bin scrape_retry
use dotenvy::dotenv;
use std::io::Write;
use tracing::Level;

use kult_browser_backend_rust::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let scraper = BrightDataPostScraper::new();
    let data_dir = std::path::PathBuf::from("data");

    // ─── TikTok (retry with a popular recent post) ───
    tracing::info!("=== Retrying TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@maboroshidotorg/video/7471447382555218218".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("tiktok.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "TikTok scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "TikTok scrape failed again"),
    }

    // ─── Reddit (retry) ───
    tracing::info!("=== Retrying Reddit ===");
    match scraper
        .get_reddit_posts(vec![
            "https://www.reddit.com/r/gaming/comments/1igzkrn/what_games_have_the_best_photo_modes/"
                .to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("reddit.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Reddit scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "Reddit scrape failed again"),
    }

    // ─── LinkedIn (retry) ───
    tracing::info!("=== Retrying LinkedIn ===");
    match scraper
        .get_linkedin_posts(vec![
            "https://www.linkedin.com/posts/bradsmi_microsoft-innovation-hubis-launching-new-activity-7430402300290105344-4GZ1"
                .to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("linkedin.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "LinkedIn scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "LinkedIn scrape failed again"),
    }

    tracing::info!("=== Retry complete! ===");
    Ok(())
}
