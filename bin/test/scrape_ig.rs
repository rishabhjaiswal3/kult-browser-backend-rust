use dotenvy::dotenv;
use std::io::Write;
use tracing::Level;
use kult_browser_backend_rust::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let scraper = BrightDataPostScraper::new();
    tracing::info!("=== Scraping Instagram (NatGeo) ===");
    match scraper.get_instagram_posts(vec!["https://www.instagram.com/p/DU_mA6Zj3Ep/".to_string()]).await {
        Ok(posts) => {
            let json = serde_json::to_string_pretty(&posts)?;
            let mut f = std::fs::File::create("data/instagram.json")?;
            f.write_all(json.as_bytes())?;
            tracing::info!(count = posts.len(), "Instagram saved");
        }
        Err(e) => tracing::error!(error = %e, "Instagram failed"),
    }
    Ok(())
}
