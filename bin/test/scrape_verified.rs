/// Scrape verified live posts with hashtags and external URLs.
/// All URLs found via browser search and confirmed to be live public posts.
///
/// Run: cargo run --bin scrape_verified
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

    // ─── 1. Twitter — Fortnite post with #FortniteNightmare #Fortnite hashtags ───
    tracing::info!("=== 1/7 Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/FortniteGame/status/1844005828453470656".to_string(),
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

    // ─── 2. Instagram — Mortal Kombat reel with #mortalkombat #gaming hashtags ───
    tracing::info!("=== 2/7 Instagram ===");
    match scraper
        .get_instagram_posts(vec![
            "https://www.instagram.com/reel/DFFaIiWC8Xk/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("instagram.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram OK");
        }
        Err(e) => tracing::error!(error = %e, "Instagram FAIL"),
    }

    // ─── 3. TikTok — @gaming video with #gaming hashtags ───
    tracing::info!("=== 3/7 TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@gaming/video/7331575459343050027".to_string(),
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

    // ─── 4. Facebook — RTXrush gaming post with #ArmyOfTwo #gaming hashtags ───
    tracing::info!("=== 4/7 Facebook ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/onlythebrave87/posts/pfbid02FQxhR4RxLxvSBEJrignWe7mQ77c16MiF3egaxKuCusve9A2PR8w2eRrURtGWWhh1l".to_string(),
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

    // ─── 5. Reddit — r/gaming link post with external URL ───
    tracing::info!("=== 5/7 Reddit ===");
    match scraper
        .get_reddit_posts(vec![
            "https://www.reddit.com/r/gaming/comments/1rd7pos/the_super_mario_galaxy_movie_description_appears/".to_string(),
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

    // ─── 6. LinkedIn — #indiegame #gamedev post ───
    tracing::info!("=== 6/7 LinkedIn ===");
    match scraper
        .get_linkedin_posts(vec![
            "https://www.linkedin.com/posts/mateo-covic-71391a1b2_indiegame-gamedev-indiedev-activity-7431446264015273984-d-go".to_string(),
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

    // ─── 7. Pinterest — keep existing (has 9 hashtags already) ───
    tracing::info!("Pinterest: keeping existing data (already has 9 hashtags)");

    tracing::info!("=== All done! ===");
    Ok(())
}
