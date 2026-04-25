use mongodb::bson::DateTime;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::CONFIG;
use crate::onchain::model::{ActivityType, OnchainActivityJob, OnchainActivityStatus};
use crate::onchain::repository::OnchainActivityRepository;

#[derive(Clone)]
pub struct OnchainActivityService {
    repository: OnchainActivityRepository,
}

#[derive(Debug, Clone)]
pub struct RecordActivityInput {
    pub user_wallet: String,
    pub activity_type: ActivityType,
    pub moment_id: String,
    pub entity_id: String,
    pub metadata_hash: String,
}

impl OnchainActivityService {
    pub fn new(repository: OnchainActivityRepository) -> Self {
        Self { repository }
    }

    pub async fn enqueue_activity(&self, input: RecordActivityInput) -> Result<(), String> {
        let user_wallet = input.user_wallet.trim().to_string();
        let moment_id = input.moment_id.trim().to_string();
        let entity_id = input.entity_id.trim().to_string();
        let metadata_hash = normalize_bytes32(&input.metadata_hash);
        let activity_id = activity_id(
            CONFIG.onchain.chain_id,
            &user_wallet,
            input.activity_type,
            &moment_id,
            &entity_id,
        );
        let now = DateTime::now();

        let job = OnchainActivityJob {
            id: None,
            activity_id,
            user_wallet,
            activity_type: input.activity_type,
            moment_id,
            entity_id,
            metadata_hash,
            status: OnchainActivityStatus::Pending,
            tx_hash: None,
            attempts: 0,
            last_error: None,
            created_at: now,
            updated_at: now,
        };

        match self.repository.create_pending(job).await {
            Ok(()) => Ok(()),
            Err(e) if e == "duplicate activity job" => {
                tracing::debug!("Onchain activity job already exists; skipping duplicate");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

pub fn activity_id(
    chain_id: u64,
    user_wallet: &str,
    activity_type: ActivityType,
    moment_id: &str,
    entity_id: &str,
) -> String {
    hash_bytes32(&format!(
        "{}:{}:{}:{}:{}",
        chain_id,
        user_wallet.trim(),
        activity_type.contract_value(),
        moment_id.trim(),
        entity_id.trim()
    ))
}

pub fn metadata_hash<T: Serialize>(metadata: &T) -> String {
    let encoded = serde_json::to_string(metadata).unwrap_or_else(|_| "{}".to_string());
    hash_bytes32(&encoded)
}

pub fn hash_bytes32(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("0x{}", to_hex(&digest))
}

pub fn normalize_bytes32(value: &str) -> String {
    let value = value.trim();
    if value.starts_with("0x") && value.len() == 66 {
        value.to_string()
    } else {
        hash_bytes32(value)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
