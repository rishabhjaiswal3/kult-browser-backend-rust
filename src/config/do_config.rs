// src/config/do_config.rs
// DigitalOcean configuration

use std::env;

/// DigitalOcean Spaces configuration
#[derive(Debug, Clone)]
pub struct DoConfig {
    /// Access Key ID
    pub access_key: String,
    /// Secret Access Key
    pub secret_key: String,
    /// Spaces Endpoint (e.g., https://nyc3.digitaloceanspaces.com)
    pub endpoint: String,
    /// Region (e.g., nyc3)
    pub region: String,
    /// Bucket Name
    pub bucket: String,
    /// Temp directory for downloaded files
    pub download_tmp_dir: String,
    /// Expiration time for presigned URLs in seconds
    pub presigned_expiration: u64,
    /// Optional upload path prefix (e.g., "moments/")
    pub upload_path: String,
}

impl DoConfig {
    /// Load DO config from environment variables.
    pub fn from_env() -> Self {
        Self {
            access_key: env::var("DO_SPACES_KEY").expect("DO_SPACES_KEY must be set"),
            secret_key: env::var("DO_SPACES_SECRET").expect("DO_SPACES_SECRET must be set"),
            endpoint: env::var("DO_SPACES_ENDPOINT").expect("DO_SPACES_ENDPOINT must be set"),
            region: env::var("DO_SPACES_REGION").expect("DO_SPACES_REGION must be set"),
            bucket: env::var("MOMENTS_DO_SPACES_BUCKET")
                .expect("MOMENTS_DO_SPACES_BUCKET must be set"),
            download_tmp_dir: env::var("MOMENTS_DOWNLOAD_TMP_DIR")
                .unwrap_or_else(|_| "/tmp/moments".to_string()),
            presigned_expiration: env::var("MOMENTS_DO_SPACES_PRESIGNED_EXPIRATION")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .expect("MOMENTS_DO_SPACES_PRESIGNED_EXPIRATION must be a number"),
            upload_path: env::var("MOMENTS_UPLOAD_PATH").unwrap_or_default(),
        }
    }

    /// Resolve upload prefix used for moments assets.
    /// Falls back to "moments" when config is missing/empty.
    pub fn effective_upload_path(&self) -> String {
        let normalized = self.upload_path.trim().trim_matches('/');
        if normalized.is_empty() {
            "moments".to_string()
        } else {
            normalized.to_string()
        }
    }
}
