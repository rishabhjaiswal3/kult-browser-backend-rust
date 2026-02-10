// src/external/digital_ocean/spaces/download.rs
// Download files from DigitalOcean Spaces URLs to local temp path

use std::path::{Path, PathBuf};

use reqwest::blocking::Client;

use crate::config::CONFIG;

/// Result of a successful download.
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Local file path where the downloaded file is saved
    pub local_path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
}

/// Download a file from a DO Spaces URL to a local temp directory.
///
/// The file is saved to `/tmp/moments/<filename>` extracted from the URL.
/// Creates the directory if it doesn't exist.
///
/// # Arguments
/// * `do_url` - Full DigitalOcean Spaces URL (e.g., `https://bucket.sgp1.digitaloceanspaces.com/moments/file.gif`)
///
/// # Returns
/// `DownloadResult` with local path and size, or error string.
pub fn download_file(do_url: &str) -> Result<DownloadResult, String> {
    tracing::info!(url = %do_url, "Downloading file from DO Spaces");

    // Extract filename from URL
    let filename = extract_filename(do_url)?;

    // Use configured temp directory
    let tmp_dir = Path::new(&CONFIG.do_spaces.download_tmp_dir);
    std::fs::create_dir_all(tmp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let local_path = tmp_dir.join(&filename);

    // Download
    let client = Client::new();
    let response = client
        .get(do_url)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "DO Spaces returned HTTP {}: {}",
            response.status(),
            do_url
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let size_bytes = bytes.len() as u64;

    // Write to file
    std::fs::write(&local_path, &bytes)
        .map_err(|e| format!("Failed to write file to {}: {}", local_path.display(), e))?;

    tracing::info!(
        path = %local_path.display(),
        size_bytes = size_bytes,
        "Download complete"
    );

    Ok(DownloadResult {
        local_path,
        size_bytes,
    })
}

/// Clean up a downloaded temp file.
pub fn cleanup(path: &Path) {
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
            tracing::warn!(path = %path.display(), error = %e, "Failed to cleanup temp file");
        } else {
            tracing::debug!(path = %path.display(), "Cleaned up temp file");
        }
    }
}

/// Extract filename from a URL.
/// e.g., `https://bucket.sgp1.digitaloceanspaces.com/moments/abc.gif` → `abc.gif`
fn extract_filename(url: &str) -> Result<String, String> {
    url.rsplit('/')
        .next()
        .filter(|f| !f.is_empty())
        .map(|f| {
            // Remove query params if any
            f.split('?').next().unwrap_or(f).to_string()
        })
        .ok_or_else(|| format!("Could not extract filename from URL: {}", url))
}
