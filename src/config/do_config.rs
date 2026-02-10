// src/config/do_config.rs
// DigitalOcean configuration

use std::env;

/// DigitalOcean Spaces configuration
#[derive(Debug, Clone)]
pub struct DoConfig {
    /// Temp directory for downloaded files
    pub download_tmp_dir: String,
}

impl DoConfig {
    /// Load DO config from environment variables.
    pub fn from_env() -> Self {
        Self {
            download_tmp_dir: env::var("DO_DOWNLOAD_TMP_DIR")
                .unwrap_or_else(|_| "/tmp/moments".to_string()),
        }
    }
}
