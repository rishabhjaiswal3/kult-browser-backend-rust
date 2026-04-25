// src/config/onchain_config.rs

use std::env;

#[derive(Debug, Clone)]
pub struct OnchainConfig {
    pub enabled: bool,
    pub rpc_url: String,
    pub chain_id: u64,
    pub activity_contract: Option<String>,
    pub relayer_private_key: Option<String>,
    pub confirmations: u64,
    pub poll_interval_secs: u64,
    pub max_retries: u32,
}

impl OnchainConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: parse_bool("ONCHAIN_ENABLED", false),
            rpc_url: env::var("ONCHAIN_RPC_URL")
                .or_else(|_| env::var("ZG_RPC_URL"))
                .unwrap_or_else(|_| "https://evmrpc.0g.ai/".to_string()),
            chain_id: env::var("ONCHAIN_CHAIN_ID")
                .unwrap_or_else(|_| "16661".to_string())
                .parse()
                .expect("ONCHAIN_CHAIN_ID must be a valid u64"),
            activity_contract: optional_env("ONCHAIN_ACTIVITY_CONTRACT"),
            relayer_private_key: optional_env("ONCHAIN_RELAYER_PRIVATE_KEY"),
            confirmations: env::var("ONCHAIN_CONFIRMATIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("ONCHAIN_CONFIRMATIONS must be a valid u64"),
            poll_interval_secs: env::var("ONCHAIN_POLL_INTERVAL_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("ONCHAIN_POLL_INTERVAL_SECS must be a valid u64"),
            max_retries: env::var("ONCHAIN_MAX_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("ONCHAIN_MAX_RETRIES must be a valid u32"),
        }
    }

    pub fn can_submit_transactions(&self) -> bool {
        self.enabled
            && self.activity_contract.is_some()
            && self.relayer_private_key.is_some()
            && !self.rpc_url.trim().is_empty()
    }
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(default)
}
