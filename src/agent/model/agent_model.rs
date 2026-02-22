// src/agent/model/agent_model.rs

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Represents an AI Agent belonging to a user matching the Dev Spec v4 requirements.
/// Collection: `ai_models`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // ==========================================
    // 1. Identity & Web3 Ownership
    // ==========================================
    /// The human user who registered the agent (references the user's wallet address)
    #[serde(rename = "ownerWallet")]
    pub owner_wallet: String,

    /// The agent's OWN wallet generated upon registration (allows autonomous actions)
    #[serde(rename = "agentWallet")]
    pub agent_wallet: String,

    /// The agent's raw private key (Required for it to sign 0G & marketplace transactions autonomously)
    #[serde(rename = "privateKey")]
    pub private_key: String,

    /// If the agent was minted as an NFT on the AIRegistry contract, this is its Token ID
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,

    // ==========================================
    // 2. KULT CORE & 0G Storage
    // ==========================================
    /// The unique 0G Storage Hash pointing to the agent's Unity gameplay configuration JSON
    /// When the game launches, the Unity Avatar downloads its brain from this hash
    #[serde(rename = "configCid", skip_serializing_if = "Option::is_none")]
    pub config_cid: Option<String>,

    /// Human-granted autonomy level (1: Observe, 2: Recommend, 3: Confirm Execute, 4: Fully Autonomous)
    #[serde(rename = "corePermissionLevel")]
    pub core_permission_level: u8,

    // ==========================================
    // 3. Matchmaking Arena & Anti-Cheat
    // ==========================================
    /// Skill rating. Uses a dynamic K-factor based on match count (Default: 1200)
    #[serde(rename = "eloRating")]
    pub elo_rating: u32,

    /// General trust score based on successful market trades & behavior (Default: 100)
    #[serde(rename = "reputationScore")]
    pub reputation_score: u32,

    /// Graduates from Level 0 (Active) to Level 4 (Banned)
    /// Flags are raised by Impossible stats (e.g. >99% accuracy or <80ms reaction time)
    #[serde(rename = "suspensionLevel")]
    pub suspension_level: u8,

    // ==========================================
    // 4. Web3 Transaction Management
    // ==========================================
    /// Tracks the number of transactions the agent has successfully fired to the blockchain.
    /// Crucial for preventing transaction collision/rejection when the agent acts autonomously.
    #[serde(rename = "nonce")]
    pub nonce: u64,

    // ==========================================
    // 5. Record Keeping
    // ==========================================
    #[serde(rename = "createdAt", default, skip_deserializing)]
    pub created_at: Option<mongodb::bson::DateTime>,

    #[serde(rename = "updatedAt", default, skip_deserializing)]
    pub updated_at: Option<mongodb::bson::DateTime>,
}

impl AgentModel {
    /// Creates a fresh agent template using the mandatory creation properties
    pub fn new(owner: String, agent_address: String, private_key: String) -> Self {
        Self {
            id: None,
            owner_wallet: owner,
            agent_wallet: agent_address,
            private_key,
            token_id: None,           // Will be filled if minted to AIRegistry later
            config_cid: None, // Will be filled once the backend uploads the initial config to 0G
            core_permission_level: 1, // Start strictly as 'Observe Only' for user safety
            elo_rating: 1200, // Base provisional rating
            reputation_score: 100, // Base clean score
            suspension_level: 0, // Level 0 = Active
            nonce: 0,         // 0 transactions sent so far
            created_at: Some(mongodb::bson::DateTime::now()),
            updated_at: Some(mongodb::bson::DateTime::now()),
        }
    }
}
