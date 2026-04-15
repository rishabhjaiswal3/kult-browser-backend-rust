/// Retry scrape for Twitter, Instagram, Facebook with posts that have BOTH tags + external links.
/// Run: cargo run --bin scrape_retry3
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

    // Twitter — Indie dev with #indiedev #gamedev + Steam link
    tracing::info!("=== Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/Ronnie_Ree/status/2020518068800160166".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("twitter_both.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Twitter OK");
        }
        Err(e) => tracing::error!(error = %e, "Twitter FAIL"),
    }

    // Instagram — Clash Royale reel with #clashroyale #gaming + deckai.app link
    tracing::info!("=== Instagram ===");
    match scraper
        .get_instagram_posts(vec![
            "https://www.instagram.com/reels/DOWbvzZEdcO/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("instagram_both.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram OK");
        }
        Err(e) => tracing::error!(error = %e, "Instagram FAIL"),
    }

    // Facebook — Kotaku with #GearsOfWar #PS5 + Gfinity article link
    tracing::info!("=== Facebook ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/kotaku/posts/1126531136015163/".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("facebook_both.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Facebook OK");
        }
        Err(e) => tracing::error!(error = %e, "Facebook FAIL"),
    }

    tracing::info!("=== Done! ===");
    Ok(())
}
