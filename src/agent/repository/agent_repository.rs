// src/agent/repository/agent_repository.rs

use crate::agent::wallet::AgentWallet;
use mongodb::{bson::doc, options::IndexOptions, Collection, Database, IndexModel};
use tracing::{error, info};

use crate::agent::model::agent_model::AgentModel;

#[derive(Clone)]
pub struct AgentRepository {
    pub collection: Collection<AgentModel>,
}

impl AgentRepository {
    pub fn new(db: &Database) -> Self {
        let collection: Collection<AgentModel> =
            db.collection(&crate::config::CONFIG.db.ai_models_collection);

        let repo = Self { collection };
        repo.init_indexes(); // Ensure critical indexes are created
        repo
    }

    /// Initializes necessary MongoDB indexes for performance and uniqueness
    fn init_indexes(&self) {
        let collection = self.collection.clone();
        tokio::spawn(async move {
            // 1. One agent per human wallet (Unique)
            let owner_index = IndexModel::builder()
                .keys(doc! { "ownerWallet": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build();

            // 2. The agent's own wallet must be globally unique
            let agent_wallet_index = IndexModel::builder()
                .keys(doc! { "agentWallet": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build();

            match collection
                .create_indexes(vec![owner_index, agent_wallet_index])
                .await
            {
                Ok(_) => info!("AgentRepository indexes created successfully"),
                Err(e) => error!("Failed to create AgentRepository indexes: {}", e),
            }
        });
    }

    /// Automatically generates a new Web3 Identity (Agent) for a user.
    /// This should be called during the Player Registration flow.
    pub async fn create_agent_for_new_user(
        &self,
        owner_wallet: &str,
    ) -> Result<AgentModel, mongodb::error::Error> {
        // 1. Generate the completely random secure EVM pair (Wallet + Private Key)
        let generated_wallet = AgentWallet::generate();

        // 2. Construct the Agent database model
        // We use lowercase to ensure standard EVM address formatting
        let new_agent = AgentModel::new(
            owner_wallet.to_lowercase(),
            generated_wallet.address.to_lowercase(),
            generated_wallet.private_key,
        );

        // 3. Insert into MongoDB
        self.collection.insert_one(&new_agent).await?;

        info!(
            owner_wallet = %owner_wallet,
            agent_wallet = %generated_wallet.address,
            "Successfully generated and stored new Web3 Agent Identity for user"
        );

        Ok(new_agent)
    }

    /// Retrieves an Agent by the human owner's wallet address
    pub async fn get_agent_by_owner(
        &self,
        owner_wallet: &str,
    ) -> Result<Option<AgentModel>, mongodb::error::Error> {
        self.collection
            .find_one(doc! { "ownerWallet": owner_wallet.to_lowercase() })
            .await
    }

    /// Retrieves an Agent by its own generated wallet address
    pub async fn get_agent_by_wallet(
        &self,
        agent_wallet: &str,
    ) -> Result<Option<AgentModel>, mongodb::error::Error> {
        self.collection
            .find_one(doc! { "agentWallet": agent_wallet.to_lowercase() })
            .await
    }
}
