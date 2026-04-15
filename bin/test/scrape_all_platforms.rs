/// Scrape real public posts across all supported platforms and save full responses to `data/`.
/// These posts were selected because they contain hashtags, external links, or both.
///
/// Run: cargo run --bin scrape_all_platforms
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

    // ─── 1. Twitter ──────────────────────────────────
    // Post with hashtags + external link
    tracing::info!("=== Scraping Twitter ===");
    match scraper
        .get_twitter_posts(vec![
            "https://x.com/elaboratelands/status/1697608866419896826".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("twitter.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Twitter scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "Twitter scrape failed"),
    }

    // ─── 2. Instagram ────────────────────────────────
    // Post with hashtags
    tracing::info!("=== Scraping Instagram ===");
    match scraper
        .get_instagram_posts(vec!["https://www.instagram.com/p/C5mWz8eomLy/".to_string()])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("instagram.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "Instagram scrape failed"),
    }

    // ─── 3. TikTok ───────────────────────────────────
    // Video with hashtags
    tracing::info!("=== Scraping TikTok ===");
    match scraper
        .get_tiktok_posts(vec![
            "https://www.tiktok.com/@khaby.lame/video/7335862561971070209".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("tiktok.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "TikTok scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "TikTok scrape failed"),
    }

    // ─── 4. Facebook ─────────────────────────────────
    // Public page post
    tracing::info!("=== Scraping Facebook ===");
    match scraper
        .get_facebook_posts(vec![
            "https://www.facebook.com/NASA/posts/pfbid0WcWD9GzWDWx6oAA4jsZL2phZ2w4egn6SYBw5LGSsheTiykpR3Zzi7u5NA5yMcgpql".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("facebook.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Facebook scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "Facebook scrape failed"),
    }

    // ─── 5. Reddit ───────────────────────────────────
    // Post with external links
    tracing::info!("=== Scraping Reddit ===");
    match scraper
        .get_reddit_posts(vec![
            "https://www.reddit.com/r/gaming/comments/1b1fsbr/new_indie_game_looks_incredible/"
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
        Err(e) => tracing::error!(error = %e, "Reddit scrape failed"),
    }

    // ─── 6. LinkedIn ─────────────────────────────────
    // Public post with hashtags + embedded links
    tracing::info!("=== Scraping LinkedIn ===");
    match scraper
        .get_linkedin_posts(vec![
            "https://www.linkedin.com/posts/sataborasu_gamedev-indiedev-unity-activity-7175436892253986816-XYZQ".to_string(),
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("linkedin.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "LinkedIn scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "LinkedIn scrape failed"),
    }

    // ─── 7. Pinterest ────────────────────────────────
    // Pin with hashtags
    tracing::info!("=== Scraping Pinterest ===");
    match scraper
        .get_pinterest_posts(vec![
            "https://www.pinterest.com/pin/99360735523397542/".to_string()
        ])
        .await
    {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create(data_dir.join("pinterest.json"))?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Pinterest scraped and saved");
        }
        Err(e) => tracing::error!(error = %e, "Pinterest scrape failed"),
    }

    tracing::info!("=== All platform scrapes complete! Check data/ for results ===");
    Ok(())
}
