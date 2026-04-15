/// Scrape the missing fields: hashtags for Twitter, external links for Instagram/Facebook/Pinterest.
/// Saves to data/<platform>_extra.json to not overwrite existing good data.
///
/// Run: cargo run --bin scrape_missing
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

    // 1. Twitter — NodwinGaming tweet WITH hashtags (#BGMI #gaming #esports)
    tracing::info!("=== 1/4 Twitter (for hashtags) ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/NodwinGaming/status/1757275963266572710".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("twitter_hashtags.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Twitter OK");
        }
        Err(e) => tracing::error!(error = %e, "Twitter FAIL"),
    }

    // 2. Instagram — CosmicByte reel with external Amazon/bit.ly links
    tracing::info!("=== 2/4 Instagram (for external link) ===");
    match scraper
        .get_instagram_posts(vec![
            "https://www.instagram.com/reel/CoZBDp2hu0H/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("instagram_exturl.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram OK");
        }
        Err(e) => tracing::error!(error = %e, "Instagram FAIL"),
    }

    // 3. Facebook — IGN post with ign.com external link
    tracing::info!("=== 3/4 Facebook (for external link) ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/ign/posts/836160878165719/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("facebook_exturl.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Facebook OK");
        }
        Err(e) => tracing::error!(error = %e, "Facebook FAIL"),
    }

    // 4. Pinterest — Spruce Eats pin with source link to thespruceeats.com
    tracing::info!("=== 4/4 Pinterest (for external link) ===");
    match scraper
        .get_pinterest_posts(vec![
            "https://www.pinterest.com/pin/991284567996114088/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("pinterest_exturl.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Pinterest OK");
        }
        Err(e) => tracing::error!(error = %e, "Pinterest FAIL"),
    }

    tracing::info!("=== All 4 scraped! ===");
    Ok(())
}
