use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use tokio::sync::watch;

use crate::config::CONFIG;
use crate::onchain::model::OnchainActivityJob;
use crate::onchain::repository::OnchainActivityRepository;

alloy::sol! {
    #[sol(rpc)]
    contract KultMomentsActivityRecorder {
        function recordActivity(
            bytes32 activityId,
            address user,
            uint8 activityType,
            bytes32 momentIdHash,
            bytes32 entityIdHash,
            bytes32 metadataHash,
            uint256 timestamp
        ) external;
    }
}

pub struct OnchainActivityWorker {
    repository: OnchainActivityRepository,
    shutdown_rx: watch::Receiver<bool>,
}

impl OnchainActivityWorker {
    pub fn new(repository: OnchainActivityRepository, shutdown_rx: watch::Receiver<bool>) -> Self {
        Self {
            repository,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        if !CONFIG.onchain.can_submit_transactions() {
            tracing::info!(
                "Onchain activity worker disabled; missing enabled flag, contract, or relayer key"
            );
            return;
        }

        tracing::info!(
            rpc_url = %CONFIG.onchain.rpc_url,
            chain_id = CONFIG.onchain.chain_id,
            "Onchain activity worker started"
        );

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        tracing::info!("Onchain activity worker received shutdown signal");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(CONFIG.onchain.poll_interval_secs)) => {
                    if let Err(e) = self.process_pending_batch().await {
                        tracing::error!(error = %e, "Onchain activity batch failed");
                    }
                }
            }
        }
    }

    async fn process_pending_batch(&self) -> Result<(), String> {
        let jobs = self.repository.find_pending(10).await?;

        for job in jobs {
            if job.attempts >= CONFIG.onchain.max_retries {
                self.repository
                    .mark_failed(&job.activity_id, "max retries exceeded")
                    .await?;
                continue;
            }

            match self.submit_job(&job).await {
                Ok(tx_hash) => {
                    self.repository
                        .mark_submitted(&job.activity_id, &tx_hash)
                        .await?;
                    self.repository.mark_confirmed(&job.activity_id).await?;
                    tracing::info!(
                        activity_id = %job.activity_id,
                        tx_hash = %tx_hash,
                        "Onchain activity submitted"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        activity_id = %job.activity_id,
                        attempts = job.attempts,
                        error = %e,
                        "Onchain activity submit failed"
                    );
                    self.repository
                        .reset_for_retry(&job.activity_id, &e)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn submit_job(&self, job: &OnchainActivityJob) -> Result<String, String> {
        let private_key = CONFIG
            .onchain
            .relayer_private_key
            .as_deref()
            .ok_or_else(|| "ONCHAIN_RELAYER_PRIVATE_KEY is missing".to_string())?;
        let contract_address = CONFIG
            .onchain
            .activity_contract
            .as_deref()
            .ok_or_else(|| "ONCHAIN_ACTIVITY_CONTRACT is missing".to_string())?;

        let signer = PrivateKeySigner::from_str(private_key)
            .map_err(|e| format!("Invalid relayer private key: {e}"))?;
        let wallet = EthereumWallet::from(signer);
        let rpc_url = CONFIG
            .onchain
            .rpc_url
            .parse()
            .map_err(|e| format!("Invalid ONCHAIN_RPC_URL: {e}"))?;
        let provider = ProviderBuilder::new().wallet(wallet).connect_http(rpc_url);
        let contract = KultMomentsActivityRecorder::new(
            Address::from_str(contract_address)
                .map_err(|e| format!("Invalid contract address: {e}"))?,
            provider,
        );

        let pending = contract
            .recordActivity(
                parse_b256(&job.activity_id)?,
                Address::from_str(&job.user_wallet)
                    .map_err(|e| format!("Invalid user wallet: {e}"))?,
                job.activity_type.contract_value(),
                hash_to_b256(&job.moment_id)?,
                hash_to_b256(&job.entity_id)?,
                parse_b256(&job.metadata_hash)?,
                U256::from(current_unix_timestamp()),
            )
            .send()
            .await
            .map_err(|e| format!("Failed to send transaction: {e}"))?;

        Ok(format!("{:#x}", pending.tx_hash()))
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hash_to_b256(value: &str) -> Result<B256, String> {
    let hash = crate::onchain::service::hash_bytes32(value);
    parse_b256(&hash)
}

fn parse_b256(value: &str) -> Result<B256, String> {
    B256::from_str(value).map_err(|e| format!("Invalid bytes32 value '{value}': {e}"))
}
