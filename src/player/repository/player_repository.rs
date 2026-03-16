use crate::config::CONFIG;
use crate::player::model::PlayerModel;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime as BsonDateTime, Document};
use mongodb::{Collection, Database};

/// Repository for Player CRUD operations.
#[derive(Clone)]
pub struct PlayerRepository {
    collection: Collection<PlayerModel>,
}

impl PlayerRepository {
    /// Create a new repository instance
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(&CONFIG.db.players_collection),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // READ OPERATIONS
    // ─────────────────────────────────────────────────────────────────────

    /// Find a player by their wallet address (case-insensitive)
    ///
    /// # Arguments
    /// * `wallet_address` - The Ethereum wallet address (will be lowercased)
    ///
    /// # Returns
    /// * `Ok(Some(PlayerModel))` if found
    /// * `Ok(None)` if not found
    pub async fn find_by_wallet(
        &self,
        wallet_address: &str,
    ) -> Result<Option<PlayerModel>, String> {
        let normalized = wallet_address.trim().to_lowercase();

        self.collection
            .find_one(doc! { "walletAddress": &normalized })
            .await
            .map_err(|e| format!("Failed to find player: {}", e))
    }

    /// Find a player by their MongoDB ObjectId
    ///
    /// # Arguments
    /// * id - The ObjectId as a hex string
    ///
    /// # Returns
    /// * `Ok(Some(PlayerModel))` if found
    /// * `Ok(None)` if not found or invalid ObjectId
    pub async fn find_by_id(&self, id: &str) -> Result<Option<PlayerModel>, String> {
        let oid = ObjectId::parse_str(id).map_err(|_| format!("Invalid ObjectId: {}", id))?;

        self.collection
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| format!("Failed to find player by id: {}", e))
    }

    /// Check if a wallet address is already registered
    pub async fn exists_by_wallet(&self, wallet_address: &str) -> Result<bool, String> {
        let normalized = wallet_address.trim().to_lowercase();

        let count = self
            .collection
            .count_documents(doc! { "walletAddress": &normalized })
            .await
            .map_err(|e| format!("Failed to check existence: {}", e))?;

        Ok(count > 0)
    }

    // ─────────────────────────────────────────────────────────────────────
    // WRITE OPERATIONS
    // ─────────────────────────────────────────────────────────────────────

    /// Create a new player.
    ///
    /// # Arguments
    /// * `wallet_address` - Ethereum wallet address
    /// * `name` - Display name
    /// * `metadata` - Optional arbitrary metadata
    ///
    /// # Returns
    /// * The created PlayerModel with its new ObjectId
    pub async fn create(
        &self,
        wallet_address: &str,
        name: &str,
        metadata: Option<Document>,
    ) -> Result<PlayerModel, String> {
        let now = BsonDateTime::now();
        let normalized_wallet = wallet_address.trim().to_lowercase();

        let player = PlayerModel {
            id: None, // MongoDB will generate
            wallet_address: normalized_wallet,
            name: name.trim().to_string(),
            metadata,
            referral_code: None,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let result = self
            .collection
            .insert_one(&player)
            .await
            .map_err(|e| format!("Failed to create player: {}", e))?;

        // Return with the inserted ID
        Ok(PlayerModel {
            id: result.inserted_id.as_object_id(),
            ..player
        })
    }

    /// Find or create a player (upsert pattern for login).
    ///
    /// This is the primary method used during login:
    /// - If player exists: return existing player
    /// - If player doesn't exist: create new player with provided details
    ///
    /// # Arguments
    /// * `wallet_address` - Ethereum wallet address
    /// * `name` - Display name (only used for creation)
    /// * `metadata` - Optional metadata (only used for creation)
    ///
    /// # Returns
    /// * [(PlayerModel, bool)](cci:1://file:///Users/ankurgangwar/Dev/fl/full_stack/browser-deployed/kult-browser-backend-rust/src/leaderboard/repository/game_leaderboard_config_repository.rs:10:4-14:5) - The player and whether it was newly created
    pub async fn find_or_create(
        &self,
        wallet_address: &str,
        name: &str,
        metadata: Option<Document>,
    ) -> Result<(PlayerModel, bool), String> {
        // First, try to find existing player
        if let Some(existing) = self.find_by_wallet(wallet_address).await? {
            return Ok((existing, false)); // false = not newly created
        }

        // Create new player
        let player = self.create(wallet_address, name, metadata).await?;
        Ok((player, true)) // true = newly created
    }

    /// Update a player's display name.
    ///
    /// # Arguments
    /// * `wallet_address` - The player's wallet address
    /// * `new_name` - The new display name
    ///
    /// # Returns
    /// * `Ok(Some(PlayerModel))` - Updated player if found
    /// * `Ok(None)` - If player not found
    pub async fn update_name(
        &self,
        wallet_address: &str,
        new_name: &str,
    ) -> Result<Option<PlayerModel>, String> {
        let normalized = wallet_address.trim().to_lowercase();
        let trimmed_name = new_name.trim();

        // Validate name is not empty
        if trimmed_name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        let result = self
            .collection
            .find_one_and_update(
                doc! { "walletAddress": &normalized },
                doc! {
                    "$set": {
                        "name": trimmed_name,
                        "updatedAt": BsonDateTime::now(),
                    }
                },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| format!("Failed to update name: {}", e))?;

        Ok(result)
    }

    /// Update a player's metadata.
    ///
    /// # Arguments
    /// * `wallet_address` - The player's wallet address
    /// * `metadata` - The new metadata document (replaces existing)
    pub async fn update_metadata(
        &self,
        wallet_address: &str,
        metadata: Document,
    ) -> Result<Option<PlayerModel>, String> {
        let normalized = wallet_address.trim().to_lowercase();

        let result = self
            .collection
            .find_one_and_update(
                doc! { "walletAddress": &normalized },
                doc! {
                    "$set": {
                        "metadata": metadata,
                        "updatedAt": BsonDateTime::now(),
                    }
                },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| format!("Failed to update metadata: {}", e))?;

        Ok(result)
    }

    // ─────────────────────────────────────────────────────────────────────
    // BULK / QUERY OPERATIONS (for admin/analytics)
    // ─────────────────────────────────────────────────────────────────────

    /// Get total player count
    pub async fn count_all(&self) -> Result<u64, String> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| format!("Failed to count players: {}", e))
    }

    /// Find multiple players by wallet addresses
    ///
    /// # Arguments
    /// * `wallet_addresses` - List of wallet addresses to find
    ///
    /// # Returns
    /// * Vector of found players (may be fewer than input if some not found)
    pub async fn find_many_by_wallets(
        &self,
        wallet_addresses: &[String],
    ) -> Result<Vec<PlayerModel>, String> {
        let normalized: Vec<String> = wallet_addresses
            .iter()
            .map(|w| w.trim().to_lowercase())
            .collect();

        let cursor = self
            .collection
            .find(doc! { "walletAddress": { "$in": &normalized } })
            .await
            .map_err(|e| format!("Failed to find players: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect players: {}", e))
    }

    // ─────────────────────────────────────────────────────────────────────
    // REFERRAL OPERATIONS
    // ─────────────────────────────────────────────────────────────────────

    /// Find a player by their referral code
    pub async fn find_by_referral_code(&self, code: &str) -> Result<Option<PlayerModel>, String> {
        self.collection
            .find_one(doc! { "referralCode": code })
            .await
            .map_err(|e| format!("Failed to find player by referral code: {}", e))
    }

    /// Set a player's referral code
    pub async fn set_referral_code(
        &self,
        wallet_address: &str,
        code: &str,
    ) -> Result<Option<PlayerModel>, String> {
        let normalized = wallet_address.trim().to_lowercase();

        let result = self
            .collection
            .find_one_and_update(
                doc! { "walletAddress": &normalized },
                doc! { "$set": { "referralCode": code } },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| format!("Failed to update referral code: {}", e))?;

        Ok(result)
    }
}
