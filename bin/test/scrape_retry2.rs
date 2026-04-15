/// Final retry for Twitter, TikTok, Reddit with known-working public post URLs.
///
/// Run: cargo run --bin scrape_retry2
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

    // ─── Twitter: PlayStation post (known to work) ───
    tracing::info!("=== Scraping Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/PlayStation/status/2024484765882339769".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("twitter.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Twitter saved");
        }
        Err(e) => tracing::error!(error = %e, "Twitter failed"),
    }

    // ─── TikTok: Highland Bros post (known to work) ───
    tracing::info!("=== Scraping TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("tiktok.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "TikTok saved");
        }
        Err(e) => tracing::error!(error = %e, "TikTok failed"),
    }

    // ─── Reddit: Popular post from r/pics (known to work) ───
    tracing::info!("=== Scraping Reddit ===");
    match scraper
        .get_reddit_posts(vec![
            "https://www.reddit.com/r/pics/comments/1rawh1j/figure_skater_alysa_liu_at_peak_bliss/"
                .to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("reddit.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Reddit saved");
        }
        Err(e) => tracing::error!(error = %e, "Reddit failed"),
    }

    tracing::info!("=== Retry 2 complete! ===");
    Ok(())
}
