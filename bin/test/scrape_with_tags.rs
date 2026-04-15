/// Scrape posts that are known to have hashtags AND/OR external links.
/// These are carefully selected real public posts where the fields are populated.
///
/// Run: cargo run --bin scrape_with_tags
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

    // ─── 1. Twitter — Epic Games tweet with #gaming hashtags + external URL ───
    tracing::info!("=== 1/7 Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/EpicGames/status/1879931532275466563".to_string()
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

    // ─── 2. Instagram — Reels/post with gaming hashtags ───
    tracing::info!("=== 2/7 Instagram ===");
    match scraper
        .get_instagram_posts(vec!["https://www.instagram.com/p/DGhxWkwP3GU/".to_string()])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("instagram.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram OK");
        }
        Err(e) => tracing::error!(error = %e, "Instagram FAIL"),
    }

    // ─── 3. TikTok — gaming video with hashtags ───
    tracing::info!("=== 3/7 TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@epicgames/video/7460949102310327595".to_string(),
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

    // ─── 4. Facebook — gaming page post with hashtags ───
    tracing::info!("=== 4/7 Facebook ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/IGN/posts/pfbid0h7nnRqt3p3x5QXQG7pBVnXgbqTQshpTVbLUJvqHMbZxsjPEjhZrcPfSb2q3nRz1l".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("facebook.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Facebook OK");
        }
        Err(e) => tracing::error!(error = %e, "Facebook FAIL"),
    }

    // ─── 5. Reddit — post with external link ───
    tracing::info!("=== 5/7 Reddit ===");
    match scraper
        .get_reddit_posts(vec![
            "https://www.reddit.com/r/Games/comments/1i76pxu/steam_broke_its_concurrent_user_record_again/".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("reddit.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Reddit OK");
        }
        Err(e) => tracing::error!(error = %e, "Reddit FAIL"),
    }

    // ─── 6. LinkedIn — post with hashtags + embedded links ───
    tracing::info!("=== 6/7 LinkedIn ===");
    match scraper
        .get_linkedin_posts(vec![
            "https://www.linkedin.com/posts/timsweeney_epicgames-unrealengine-gamedev-activity-7288966019614691328-abCD".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("linkedin.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "LinkedIn OK");
        }
        Err(e) => tracing::error!(error = %e, "LinkedIn FAIL"),
    }

    // ─── 7. Pinterest — pin with hashtags ───
    tracing::info!("=== 7/7 Pinterest ===");
    match scraper
        .get_pinterest_posts(vec![
            "https://www.pinterest.com/pin/820710732116792285/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("pinterest.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Pinterest OK");
        }
        Err(e) => tracing::error!(error = %e, "Pinterest FAIL"),
    }

    tracing::info!("=== All 7 platforms done! ===");
    Ok(())
}
