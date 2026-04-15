/// Test the full post verification pipeline.
///
/// Part 1: Unit tests for PostValidator (no API calls)
/// Part 2: Integration test — scrape real posts via BD, normalize, validate
///
/// Run: cargo run --bin test_post_verification
use dotenvy::dotenv;
use tracing::Level;

use kult_browser_backend_rust::external::bright_data::scrapers::post_scrapers::BrightDataPostScraper;
use kult_browser_backend_rust::external::bright_data::scrapers::scraped_post::ScrapedPostData;
use kult_browser_backend_rust::moments::social_media::model::platform::Platform;
use kult_browser_backend_rust::moments::social_media::service::post_validator::{
    PostValidator, ValidationReason,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║     POST VERIFICATION PIPELINE - FULL TEST      ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    // ═══════════════════════════════════════
    // PART 1: UNIT TESTS (PostValidator only)
    // ═══════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  PART 1: PostValidator UNIT TESTS (no API calls)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut passed = 0u32;
    let mut failed = 0u32;

    // Test 1: Hashtag match — "kultgames"
    {
        let post = make_post(vec!["kultgames"], vec![], "some text");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Hashtag 'kultgames' -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 2: Hashtag match — "kult.games"
    {
        let post = make_post(vec!["kult.games"], vec![], "some text");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Hashtag 'kult.games' -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 3: Hashtag match — "kult"
    {
        let post = make_post(vec!["kult"], vec![], "some text");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Hashtag 'kult' -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 4: Hashtag case insensitive (normalizer lowercases, but test validator)
    {
        let post = make_post(vec!["KultGames"], vec![], "some text");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Hashtag 'KultGames' (case) -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 5: No match — random hashtags
    {
        let post = make_post(
            vec!["gaming", "indiedev", "gamedev"],
            vec![],
            "just a normal post",
        );
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Random hashtags -> Invalid",
            valid,
            false,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 6: URL field match
    {
        let post = make_post(
            vec![],
            vec!["https://kult.games/m/123?ref=abc"],
            "some text",
        );
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "URL 'kult.games' in external_urls -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 7: URL no match
    {
        let post = make_post(vec![], vec!["https://example.com"], "some text");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "URL 'example.com' -> Invalid",
            valid,
            false,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 8: Text regex — kult.games URL in description
    {
        let post = make_post(
            vec![],
            vec![],
            "Check out https://kult.games/m/456 for the moment!",
        );
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Text contains kult.games URL -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 9: Text regex — @kultgames mention
    {
        let post = make_post(vec![], vec![], "Shoutout to @kultgames for this moment!");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Text contains @kultgames -> Valid",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 10: BD error response
    {
        let mut post = make_post(vec![], vec![], "");
        post.error = Some("Post isn't available".to_string());
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "BD error response -> Invalid (ScraperError)",
            valid,
            false,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 11: Empty post — no signals
    {
        let post = make_post(vec![], vec![], "");
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "Empty post -> Invalid (NoMatch)",
            valid,
            false,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    // Test 12: Multiple signals — hashtag wins (short-circuit)
    {
        let post = make_post(
            vec!["kultgames"],
            vec!["https://kult.games"],
            "Check @kultgames",
        );
        let (valid, reason) = PostValidator::validate(&post);
        check(
            "All 3 signals -> Valid (hashtag wins)",
            valid,
            true,
            &reason,
            &mut passed,
            &mut failed,
        );
    }

    println!("\n  -- Unit Test Results --");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);

    // ═══════════════════════════════════════
    // PART 2: INTEGRATION TESTS (live BD scrapes)
    // ═══════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  PART 2: INTEGRATION TESTS (live BD scrape -> normalize -> validate)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let scraper = BrightDataPostScraper::new();

    // These are real public posts from the existing snapshot set under data/.
    // They are not Kult posts, so the expected result is Invalid / NoMatch.
    let test_cases: Vec<(&str, Platform, &str)> = vec![
        (
            "Twitter",
            Platform::Twitter,
            "https://x.com/Ronnie_Ree/status/2020518068800160166",
        ),
        (
            "Instagram",
            Platform::Instagram,
            "https://www.instagram.com/reel/CoZBDp2hu0H/",
        ),
        (
            "TikTok",
            Platform::TikTok,
            "https://www.tiktok.com/@extrahighlandbros/video/7609101937220308255",
        ),
        (
            "Facebook",
            Platform::Facebook,
            "https://www.facebook.com/kotaku/posts/1126531136015163/",
        ),
        (
            "Reddit",
            Platform::Reddit,
            "https://www.reddit.com/r/gaming/comments/1rdppqz/1_million_in_debt_devs_on_handheld_tony_hawks_pro/",
        ),
        (
            "LinkedIn",
            Platform::LinkedIn,
            "https://www.linkedin.com/posts/poki_gamedev-webgaming-monetisation-activity-7414576322225778688-o4T2",
        ),
        (
            "Pinterest",
            Platform::Pinterest,
            "https://www.pinterest.com/pin/1110418851898928764/",
        ),
    ];

    for (name, platform, url) in test_cases {
        println!("  -- {} --", name);
        match scraper.scrape_post(&platform, url).await {
            Ok(scraped) => {
                let (is_valid, reason) = PostValidator::validate(&scraped);
                println!("    Hashtags:      {:?}", scraped.hashtags);
                println!("    External URLs: {:?}", scraped.external_urls);
                println!(
                    "    Text (first 80): {:?}",
                    &scraped.text_content.chars().take(80).collect::<String>()
                );
                println!("    Likes:         {}", scraped.likes);
                println!("    Error:         {:?}", scraped.error);
                println!("    Is Valid:      {}", is_valid);
                println!("    Reason:        {}", reason);

                if is_valid {
                    failed += 1;
                    println!("    FAIL: expected Invalid for a non-Kult post, got Valid\n");
                } else {
                    passed += 1;
                    println!("    PASS: scrape_post -> normalize -> validate works\n");
                }
            }
            Err(e) => {
                failed += 1;
                println!("    FAIL: scrape failed: {}\n", e);
            }
        }
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║              ALL TESTS COMPLETE                 ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        return Err(format!("{failed} post verification test(s) failed").into());
    }

    Ok(())
}

/// Helper: create a ScrapedPostData for unit testing
fn make_post(hashtags: Vec<&str>, urls: Vec<&str>, text: &str) -> ScrapedPostData {
    ScrapedPostData {
        hashtags: hashtags.into_iter().map(|s| s.to_string()).collect(),
        external_urls: urls.into_iter().map(|s| s.to_string()).collect(),
        text_content: text.to_string(),
        likes: 42,
        error: None,
        raw: serde_json::json!({}),
    }
}

/// Helper: check a test result
fn check(
    name: &str,
    actual: bool,
    expected: bool,
    reason: &ValidationReason,
    passed: &mut u32,
    failed: &mut u32,
) {
    if actual == expected {
        *passed += 1;
        println!("  PASS {} - reason: {}", name, reason);
    } else {
        *failed += 1;
        println!(
            "  FAIL {} - expected {}, got {} - reason: {}",
            name, expected, actual, reason
        );
    }
}
