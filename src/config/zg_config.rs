// src/config/zg_config.rs
// 0G Storage configuration

use std::env;

/// 0G Storage CLI configuration
#[derive(Debug, Clone)]
pub struct ZgConfig {
    /// Path to the 0g-storage-client binary
    pub binary_path: String,
    /// Blockchain RPC endpoint (--url)
    pub rpc_url: String,
    /// Private key for signing transactions (--key)
    pub private_key: String,
    /// Storage indexer endpoint (--indexer)
    pub indexer_url: String,
    /// RPC timeout (--rpc-timeout)
    pub rpc_timeout: String,
    /// RPC retry count (--rpc-retry-count)
    pub rpc_retry_count: u32,
    /// RPC retry interval (--rpc-retry-interval)
    pub rpc_retry_interval: String,
    /// Optional public gateway URL template for 0G file access.
    pub gateway_url: Option<String>,
    /// Optional explorer transaction URL template for 0G chain txs.
    pub explorer_tx_url: Option<String>,
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
        }
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
