/// Scrape posts verified to have BOTH hashtags AND external links.
/// Saves to data/<platform>_both.json
///
/// Run: cargo run --bin scrape_both
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

    // 1. Twitter — GameSpot with #HonestGameTrailer + external link
    tracing::info!("=== 1/6 Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/GameSpot/status/2026452944091050442".to_string()
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

    // 2. Instagram — gaming post with hashtags + Steam link in caption
    tracing::info!("=== 2/6 Instagram ===");
    match scraper
        .get_instagram_posts(vec!["https://www.instagram.com/p/C-0p7CqS6Yy/".to_string()])
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

    // 3. Facebook — GameSpot Xbox post with hashtags + shared link
    tracing::info!("=== 3/6 Facebook ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/GameSpot/posts/1484453519714855".to_string(),
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

    // 4. LinkedIn — Poki with #GameDev + monetisation article link
    tracing::info!("=== 4/6 LinkedIn ===");
    match scraper.get_linkedin_posts(vec![
        "https://www.linkedin.com/posts/poki_gamedev-webgaming-monetisation-activity-7414576322225778688-o4T2".to_string(),
    ]).await {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("linkedin_both.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "LinkedIn OK");
        }
        Err(e) => tracing::error!(error = %e, "LinkedIn FAIL"),
    }

    // 5. Pinterest — Xbox Showcase pin with hashtags + source link
    tracing::info!("=== 5/6 Pinterest ===");
    match scraper
        .get_pinterest_posts(vec![
            "https://www.pinterest.com/pin/1110418851898928764/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("pinterest_both.json"))?
                .write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Pinterest OK");
        }
        Err(e) => tracing::error!(error = %e, "Pinterest FAIL"),
    }

    // 6. Reddit — r/gaming link post with external article
    tracing::info!("=== 6/6 Reddit ===");
    match scraper.get_reddit_posts(vec![
        "https://www.reddit.com/r/gaming/comments/1rdppqz/1_million_in_debt_devs_on_handheld_tony_hawks_pro/".to_string(),
    ]).await {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            std::fs::File::create(data_dir.join("reddit_both.json"))?.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Reddit OK");
        }
        Err(e) => tracing::error!(error = %e, "Reddit FAIL"),
    }

    tracing::info!("=== All 6 done! ===");
    Ok(())
}
