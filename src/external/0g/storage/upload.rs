// src/external/0g/storage/upload.rs
// Upload files to 0G Storage via the 0g-storage-client CLI binary

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

    tracing::info!(file = %file_path, "Starting 0G upload");

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

    // The CLI logs to stderr (logrus format)
    let combined = format!("{}\n{}", stdout, stderr);

    tracing::debug!(output = %combined, "0g-storage-client output");

    if !output.status.success() {
        tracing::error!(
            exit_code = ?output.status.code(),
            output = %combined,
            "0g-storage-client failed"
        );
        return Err(format!("0g-storage-client exited with: {}", output.status));
    }

    // Parse root hash from: "file uploaded, root = 0x..."
    let root_hash = parse_root_hash(&combined).ok_or_else(|| {
        tracing::error!(output = %combined, "Could not parse root hash from output");
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

/// Parse root hash from CLI output.
/// Looks for: `file uploaded, root = 0x<hash>`
fn parse_root_hash(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("file uploaded, root =") {
            // Extract everything after "root = "
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
