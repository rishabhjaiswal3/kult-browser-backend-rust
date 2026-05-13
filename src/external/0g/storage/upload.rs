// src/external/0g/storage/upload.rs
// Upload files to 0G Storage via the 0g-storage-client CLI binary

use std::path::PathBuf;
use std::process::Command;

use crate::config::CONFIG;

/// Result of a successful 0G upload.
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// The Merkle root hash — 0G's file identifier
    pub root_hash: String,
    /// The on-chain transaction hash
    pub tx_hash: Option<String>,
}

/// Upload a local file to 0G Storage.
///
/// Shells out to the `0g-storage-client` binary using the configured
/// RPC endpoint, private key, and indexer. Parses stdout/stderr for
/// the root hash and transaction hash.
///
/// Returns `UploadResult` on success, or an error string on failure.
pub fn upload_file(file_path: &str) -> Result<UploadResult, String> {
    let cfg = &CONFIG.zg;

    // Log the full command being run (private key redacted)
    tracing::info!(
        command = %format!(
            "{} upload --url {} --key <REDACTED> --indexer {} --file {} --rpc-timeout {} --rpc-retry-count {} --rpc-retry-interval {} --log-level debug --web3-log-enabled",
            cfg.binary_path, cfg.rpc_url, cfg.indexer_url, file_path,
            cfg.rpc_timeout, cfg.rpc_retry_count, cfg.rpc_retry_interval
        ),
        "Running 0G upload"
    );

    let output = Command::new(&cfg.binary_path)
        .arg("upload")
        .arg("--url")
        .arg(&cfg.rpc_url)
        .arg("--key")
        .arg(&cfg.private_key)
        .arg("--indexer")
        .arg(&cfg.indexer_url)
        .arg("--file")
        .arg(file_path)
        .arg("--rpc-timeout")
        .arg(&cfg.rpc_timeout)
        .arg("--rpc-retry-count")
        .arg(cfg.rpc_retry_count.to_string())
        .arg("--rpc-retry-interval")
        .arg(&cfg.rpc_retry_interval)
        .arg("--log-level")
        .arg("debug")
        .arg("--web3-log-enabled")
        .output()
        .map_err(|e| format!("Failed to execute 0g-storage-client: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    if !output.status.success() {
        // Extract clean error from FATA line instead of dumping everything
        let error_msg = parse_fatal_error(&combined)
            .unwrap_or_else(|| format!("0g-storage-client exited with: {}", output.status));
        tracing::error!(error = %error_msg, "0G upload failed");
        return Err(error_msg);
    }

    // Parse root hash from: "file uploaded, root = 0x..."
    let root_hash = parse_root_hash(&combined).ok_or_else(|| {
        tracing::error!("Could not parse root hash from 0g-storage-client output");
        "Could not parse root hash from 0g-storage-client output".to_string()
    })?;

    // Parse tx hash from: "Succeeded to send transaction to append log entry  hash=0x..."
    let tx_hash = parse_tx_hash(&combined);

    tracing::info!(
        root_hash = %root_hash,
        tx_hash = ?tx_hash,
        "0G upload complete"
    );

    Ok(UploadResult { root_hash, tx_hash })
}

/// Upload JSON metadata to 0G Storage by writing a temporary file and reusing the CLI path.
pub fn upload_json_value(prefix: &str, value: &serde_json::Value) -> Result<UploadResult, String> {
    let filename = format!(
        "{}-{}.json",
        sanitize_filename_prefix(prefix),
        nanoid::nanoid!()
    );
    let path: PathBuf = std::env::temp_dir().join("moments").join(filename);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create metadata temp directory: {}", e))?;
    }

    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("Failed to serialize 0G metadata JSON: {}", e))?;
    std::fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write 0G metadata temp file: {}", e))?;

    let result = upload_file(&path.to_string_lossy());
    if let Err(e) = std::fs::remove_file(&path) {
        tracing::warn!(path = %path.display(), error = %e, "Failed to remove 0G metadata temp file");
    }

    result
}

fn sanitize_filename_prefix(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "moment-metadata".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Parse the FATA (fatal) line from logrus output for a clean error message.
fn parse_fatal_error(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("FATA") {
            // Extract the error= field value
            if let Some(pos) = line.find("error=") {
                let msg = line[pos + 7..].trim().trim_matches('"').to_string();
                return Some(msg);
            }
            // Fallback: extract the message part after the log level bracket
            if let Some(pos) = line.find(']') {
                return Some(line[pos + 1..].trim().to_string());
            }
        }
    }
    None
}

/// Parse root hash from CLI output.
/// Looks for: `file uploaded, root = 0x<hash>`
fn parse_root_hash(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("file uploaded, root =") {
            if let Some(pos) = line.find("root = ") {
                let hash = line[pos + 7..].trim().to_string();
                if hash.starts_with("0x") {
                    return Some(hash);
                }
            }
        }
    }
    None
}

/// Parse transaction hash from CLI output.
/// Looks for: `Succeeded to send transaction to append log entry  hash=0x<hash>`
fn parse_tx_hash(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Succeeded to send transaction") {
            if let Some(pos) = line.find("hash=") {
                let hash = line[pos + 5..].trim().to_string();
                if hash.starts_with("0x") {
                    return Some(hash);
                }
            }
        }
    }
    None
}
