// src/external/0g/da/disperser.rs
// Client for the 0G DA HTTP disperser gateway.
//
// 0G DA is NOT the same as 0G Storage:
//   Storage  = Merkle-tree file store, retrieve-by-hash
//   DA       = Validator-attested blob dispersal, proven by batch receipts
//
// API (gRPC-gateway HTTP wrapper):
//   POST {url}/DisperseBlob          — submit blob, returns requestId
//   GET  {url}/GetBlobStatus         — poll finalization, returns receipt

use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Final proof receipt from 0G DA after a blob is finalized.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DaReceipt {
    pub request_id: String,
    pub batch_id: Option<u64>,
    pub blob_index: Option<u32>,
    /// The root hash of the DA batch — the primary on-chain anchor
    pub batch_header_hash: Option<String>,
    pub confirmation_block: Option<u64>,
    pub finalized_at: Option<String>,
}

/// Current state of a dispersed blob.
#[derive(Debug, Clone, PartialEq)]
pub enum DaBlobStatus {
    /// Submitted but not yet confirmed by validators
    Processing,
    /// Confirmed by enough validators — finalization pending
    Confirmed,
    /// Fully finalized — receipt available with batch proof
    Finalized(DaReceipt),
    /// Dispersal permanently failed
    Failed(String),
    /// Status unknown / not yet indexed
    Unknown,
}

pub struct ZgDaClient {
    client: reqwest::Client,
    disperser_url: String,
}

impl ZgDaClient {
    pub fn from_config() -> Option<Self> {
        let url = crate::config::CONFIG.zg.da_disperser_url.clone()?;
        Some(Self::new(url))
    }

    pub fn new(disperser_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            disperser_url: disperser_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Submit a blob to the 0G DA disperser. Returns the opaque requestId string.
    pub async fn disperse_blob(&self, data: &[u8]) -> Result<String, String> {
        let encoded = STANDARD.encode(data);
        let url = format!("{}/DisperseBlob", self.disperser_url);

        let body = serde_json::json!({ "data": encoded });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("0G DA disperser request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!(
                "0G DA disperser error {}: {}",
                status,
                &text[..text.len().min(400)]
            ));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse 0G DA disperser response: {}", e))?;

        // Handles both snake_case (gRPC-Gateway) and camelCase variants
        let request_id = raw
            .get("request_id")
            .or_else(|| raw.get("requestId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing request_id in DA response: {}", raw))?;

        Ok(request_id.to_string())
    }

    /// Poll the 0G DA disperser for the current blob status.
    pub async fn get_blob_status(&self, request_id: &str) -> Result<DaBlobStatus, String> {
        let url = format!("{}/GetBlobStatus", self.disperser_url);

        let response = self
            .client
            .get(&url)
            .query(&[("request_id", request_id)])
            .send()
            .await
            .map_err(|e| format!("0G DA status request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!(
                "0G DA status error {}: {}",
                status,
                &text[..text.len().min(400)]
            ));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse 0G DA status response: {}", e))?;

        // Status is an integer (protobuf enum) or a string variant
        // 0=UNKNOWN 1=PROCESSING 2=CONFIRMED 3=FINALIZED 4=FAILED 5=INSUFFICIENT_SIGNATURES
        let status_int = raw
            .get("status")
            .and_then(|s| {
                s.as_u64().or_else(|| {
                    s.as_str().and_then(|sv| match sv {
                        "PROCESSING" | "1" => Some(1),
                        "CONFIRMED" | "2" => Some(2),
                        "FINALIZED" | "3" => Some(3),
                        "FAILED" | "4" => Some(4),
                        "INSUFFICIENT_SIGNATURES" | "5" => Some(5),
                        _ => None,
                    })
                })
            })
            .unwrap_or(0);

        match status_int {
            1 => Ok(DaBlobStatus::Processing),
            2 => Ok(DaBlobStatus::Confirmed),
            3 => Ok(DaBlobStatus::Finalized(self.extract_receipt(&raw, request_id))),
            4 => {
                let err = raw
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("DA blob dispersal failed")
                    .to_string();
                Ok(DaBlobStatus::Failed(err))
            }
            5 => Ok(DaBlobStatus::Failed(
                "Insufficient validator signatures — blob not attested".to_string(),
            )),
            _ => Ok(DaBlobStatus::Unknown),
        }
    }

    fn extract_receipt(&self, raw: &serde_json::Value, request_id: &str) -> DaReceipt {
        let info = raw.get("info");
        let proof = info.and_then(|i| {
            i.get("blob_verification_proof")
                .or_else(|| i.get("blobVerificationProof"))
        });

        let batch_id = proof
            .and_then(|p| p.get("batch_id").or_else(|| p.get("batchId")))
            .and_then(|v| v.as_u64());

        let blob_index = proof
            .and_then(|p| p.get("blob_index").or_else(|| p.get("blobIndex")))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let metadata = proof.and_then(|p| {
            p.get("batch_metadata")
                .or_else(|| p.get("batchMetadata"))
        });

        // Prefer batch_header_hash, fall back to batch_root inside batch_header
        let batch_header_hash = metadata
            .and_then(|m| {
                m.get("batch_header_hash")
                    .or_else(|| m.get("batchHeaderHash"))
            })
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                metadata
                    .and_then(|m| {
                        m.get("batch_header")
                            .or_else(|| m.get("batchHeader"))
                    })
                    .and_then(|h| {
                        h.get("batch_root")
                            .or_else(|| h.get("batchRoot"))
                    })
                    .and_then(|v| v.as_str())
                    .map(String::from)
            });

        let confirmation_block = metadata
            .and_then(|m| {
                m.get("confirmation_block_number")
                    .or_else(|| m.get("confirmationBlockNumber"))
            })
            .and_then(|v| v.as_u64());

        let finalized_at = chrono::Utc::now().to_rfc3339();

        DaReceipt {
            request_id: request_id.to_string(),
            batch_id,
            blob_index,
            batch_header_hash,
            confirmation_block,
            finalized_at: Some(finalized_at),
        }
    }
}
