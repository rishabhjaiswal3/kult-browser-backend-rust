// src/bin/test_0g_upload.rs
// Live test binary for 0G Storage upload.
//
// Run:
//   cargo run --bin test_0g_upload
//   cargo run --bin test_0g_upload -- /path/to/file

use std::path::PathBuf;

use dotenvy::dotenv;
use kult_browser_backend_rust::external::storage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut temp_file: Option<PathBuf> = None;
    let file_path = if let Some(file_path) = std::env::args().nth(1) {
        file_path
    } else {
        let path = std::env::temp_dir().join(format!("test_0g_upload_{}.txt", nanoid::nanoid!(8)));
        std::fs::write(
            &path,
            format!(
                "0G upload smoke test\ncreated_at={}\n",
                chrono::Utc::now().to_rfc3339()
            ),
        )?;
        temp_file = Some(path.clone());
        path.display().to_string()
    };

    println!("=== 0G Upload Test ===");
    println!("File: {}\n", file_path);

    let result = match storage::upload_file(&file_path) {
        Ok(result) => {
            println!("\nUpload successful");
            println!("Root hash: {}", result.root_hash);
            println!("Tx hash:   {:?}", result.tx_hash);
            result
        }
        Err(e) => {
            eprintln!("\nUpload failed: {}", e);
            return Err(e.into());
        }
    };

    if let Some(path) = temp_file {
        let _ = std::fs::remove_file(path);
    }

    println!(
        "\n0G upload verified. Root hash={}, tx_hash={:?}",
        result.root_hash, result.tx_hash
    );

    Ok(())
}
