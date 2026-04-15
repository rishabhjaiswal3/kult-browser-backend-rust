// src/bin/test_do_download.rs
// Test binary for DigitalOcean Spaces download

use kult_browser_backend_rust::external::spaces;

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let do_url = std::env::args()
        .nth(1)
        .expect("Usage: test_do_download <do_spaces_url>");

    println!("=== DO Spaces Download Test ===");
    println!("URL: {}\n", do_url);

    match spaces::download_file(&do_url).await {
        Ok(result) => {
            println!("\n✅ Download successful!");
            println!("   Local path: {}", result.local_path.display());
            println!("   Size: {} bytes", result.size_bytes);

            // Cleanup
            println!("\n   Cleaning up temp file...");
            spaces::cleanup(&result.local_path);
            println!("   Done.");
        }
        Err(e) => {
            eprintln!("\n❌ Download failed: {}", e);
            std::process::exit(1);
        }
    }
}
