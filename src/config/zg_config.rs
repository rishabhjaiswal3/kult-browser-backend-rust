// src/config/zg_config.rs
// 0G Storage + Compute configuration

use std::env;

/// 0G Storage CLI + Compute configuration
#[derive(Debug, Clone)]
pub struct ZgConfig {
    // === Storage CLI ===
    pub binary_path: String,
    pub rpc_url: String,
    pub private_key: String,
    pub indexer_url: String,
    pub rpc_timeout: String,
    pub rpc_retry_count: u32,
    pub rpc_retry_interval: String,
    pub gateway_url: Option<String>,
    pub explorer_tx_url: Option<String>,

    // === DA (Data Availability disperser) ===
    /// HTTP gateway URL for the 0G DA disperser.
    /// Example: https://da-disperser.0g.ai
    pub da_disperser_url: Option<String>,

    // === Compute (OpenAI-compatible inference) ===
    /// Provider service base URL. Obtained via: 0g-compute-cli inference list-providers
    pub compute_provider_url: Option<String>,
    /// Bearer token (app-sk-<SECRET>). Obtained via: 0g-compute-cli inference get-secret --provider <ADDRESS>
    pub compute_api_key: Option<String>,
    /// Model name as reported by the provider
    pub compute_model: String,
}

impl ZgConfig {
    /// Load 0G config from environment variables.
    pub fn from_env() -> Self {
        Self {
            binary_path: env::var("ZG_BINARY_PATH")
                .unwrap_or_else(|_| "./0g-storage-client".to_string()),

            rpc_url: env::var("ZG_RPC_URL").unwrap_or_else(|_| "https://evmrpc.0g.ai/".to_string()),

            private_key: env::var("ZG_PRIVATE_KEY").unwrap_or_else(|_| {
                panic!("❌ ZG_PRIVATE_KEY is required. Set it in .env or environment.")
            }),

            indexer_url: env::var("ZG_INDEXER_URL")
                .unwrap_or_else(|_| "https://indexer-storage-turbo.0g.ai".to_string()),

            rpc_timeout: env::var("ZG_RPC_TIMEOUT").unwrap_or_else(|_| "800s".to_string()),

            rpc_retry_count: env::var("ZG_RPC_RETRY_COUNT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("ZG_RPC_RETRY_COUNT must be a valid u32"),

            rpc_retry_interval: env::var("ZG_RPC_RETRY_INTERVAL")
                .unwrap_or_else(|_| "3s".to_string()),

            gateway_url: optional_env("ZG_GATEWAY_URL"),
            explorer_tx_url: optional_env("ZG_EXPLORER_TX_URL"),

            da_disperser_url: optional_env("ZG_DA_DISPERSER_URL"),

            compute_provider_url: optional_env("ZG_COMPUTE_PROVIDER_URL"),
            compute_api_key: optional_env("ZG_COMPUTE_API_KEY"),
            compute_model: env::var("ZG_COMPUTE_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
        }
    }

    pub fn has_compute(&self) -> bool {
        self.compute_provider_url.is_some() && self.compute_api_key.is_some()
    }

    pub fn gateway_url_for_hash(&self, root_hash: &str) -> Option<String> {
        build_template_url(self.gateway_url.as_deref(), "hash", root_hash)
    }

    pub fn explorer_url_for_tx(&self, tx_hash: &str) -> Option<String> {
        build_template_url(self.explorer_tx_url.as_deref(), "txHash", tx_hash)
    }
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn build_template_url(template: Option<&str>, token: &str, value: &str) -> Option<String> {
    let template = template?.trim();
    if template.is_empty() || value.trim().is_empty() {
        return None;
    }

    let placeholder = format!("{{{}}}", token);
    if template.contains(&placeholder) {
        Some(template.replace(&placeholder, value.trim()))
    } else {
        Some(format!(
            "{}/{}",
            template.trim_end_matches('/'),
            value.trim()
        ))
    }
}
