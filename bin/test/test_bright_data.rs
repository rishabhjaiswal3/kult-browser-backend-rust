// src/bin/test_bright_data.rs
// Live Bright Data smoke test for all supported platforms.
//
// Run:
//   cargo run --bin test_bright_data

use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

use dotenvy::dotenv;

use kult_browser_backend_rust::external::bright_data::BrightDataPostScraper;
use kult_browser_backend_rust::handler::AppError;
use serde_json::Value;

const SEP: &str = "============================================================";
const OUTPUT_DIR: &str = "data/bright_data_live";

async fn run_case<Fut>(
    label: &str,
    url: &str,
    output_path: PathBuf,
    scrape_future: Fut,
) -> Result<(), String>
where
    Fut: Future<Output = Result<Vec<Value>, AppError>>,
{
    println!("\n{SEP}\n  {label}\n{SEP}");
    println!("  URL: {url}");

    let posts = scrape_future.await.map_err(|err| err.to_string())?;
    let json = serde_json::to_string_pretty(&posts).map_err(|err| err.to_string())?;
    fs::write(&output_path, json).map_err(|err| err.to_string())?;

    println!(
        "  Saved {} record(s) -> {}",
        posts.len(),
        output_path.display()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let data_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(data_dir).expect("Failed to create data dir");

    let scraper = BrightDataPostScraper::new();
    let mut failures = Vec::new();

    let twitter_url = "https://x.com/Ronnie_Ree/status/2020518068800160166";
    if let Err(err) = run_case(
        "TWITTER / X",
        twitter_url,
        data_dir.join("twitter.json"),
        scraper.get_twitter_posts(vec![twitter_url.to_string()]),
    )
    .await
    {
        failures.push(format!("Twitter/X: {err}"));
    }

    let instagram_url = "https://www.instagram.com/reel/CoZBDp2hu0H/";
    if let Err(err) = run_case(
        "INSTAGRAM",
        instagram_url,
        data_dir.join("instagram.json"),
        scraper.get_instagram_posts(vec![instagram_url.to_string()]),
    )
    .await
    {
        failures.push(format!("Instagram: {err}"));
    }

    let tiktok_url = "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255";
    if let Err(err) = run_case(
        "TIKTOK",
        tiktok_url,
        data_dir.join("tiktok.json"),
        scraper.get_tiktok_posts(vec![tiktok_url.to_string()]),
    )
    .await
    {
        failures.push(format!("TikTok: {err}"));
    }

    let facebook_url = "https://www.facebook.com/kotaku/posts/1126531136015163/";
    if let Err(err) = run_case(
        "FACEBOOK",
        facebook_url,
        data_dir.join("facebook.json"),
        scraper.get_facebook_posts(vec![facebook_url.to_string()]),
    )
    .await
    {
        failures.push(format!("Facebook: {err}"));
    }

    let reddit_url = "https://www.reddit.com/r/gaming/comments/1rdppqz/1_million_in_debt_devs_on_handheld_tony_hawks_pro/";
    if let Err(err) = run_case(
        "REDDIT",
        reddit_url,
        data_dir.join("reddit.json"),
        scraper.get_reddit_posts(vec![reddit_url.to_string()]),
    )
    .await
    {
        failures.push(format!("Reddit: {err}"));
    }

    let linkedin_url =
        "https://www.linkedin.com/posts/poki_gamedev-webgaming-monetisation-activity-7414576322225778688-o4T2";
    if let Err(err) = run_case(
        "LINKEDIN",
        linkedin_url,
        data_dir.join("linkedin.json"),
        scraper.get_linkedin_posts(vec![linkedin_url.to_string()]),
    )
    .await
    {
        failures.push(format!("LinkedIn: {err}"));
    }

    let pinterest_url = "https://www.pinterest.com/pin/1110418851898928764/";
    if let Err(err) = run_case(
        "PINTEREST",
        pinterest_url,
        data_dir.join("pinterest.json"),
        scraper.get_pinterest_posts(vec![pinterest_url.to_string()]),
    )
    .await
    {
        failures.push(format!("Pinterest: {err}"));
    }

    if !failures.is_empty() {
        eprintln!("\n{SEP}\n  FAILURES\n{SEP}");
        for failure in &failures {
            eprintln!("  - {failure}");
        }
        return Err(format!("{} Bright Data scrape(s) failed", failures.len()).into());
    }

    println!("\n{SEP}\n  ALL BRIGHT DATA PLATFORM SCRAPES PASSED\n{SEP}");
    Ok(())
}
