use crate::handler::AppError;
use crate::leaderboard::repository::GlobalLeaderboardRepository;
use crate::leaderboard::service::GameLeaderboardService;
use crate::middleware::AuthService;
use crate::player::dto::{
    GameScoreEntry, LoginRequest, LoginResponse, PlayerInfo, PlayerProfile, PlayerProfileResponse,
    UpdateNameRequest, UpdateNameResponse,
};
use crate::player::repository::PlayerRepository;
use mongodb::bson::Document;
use serde_json::Value;

use crate::agent::repository::agent_repository::AgentRepository;

use crate::referral::anti_fraud::AntiFraudService;

/// Service layer for Player operations.
#[derive(Clone)]
pub struct PlayerService {
    player_repo: PlayerRepository,
    global_lb_repo: GlobalLeaderboardRepository,
    game_lb_service: GameLeaderboardService,
    agent_repo: AgentRepository,
    anti_fraud_service: Option<std::sync::Arc<AntiFraudService>>,
}

impl PlayerService {
    pub fn new(
        player_repo: PlayerRepository,
        global_lb_repo: GlobalLeaderboardRepository,
        game_lb_service: GameLeaderboardService,
        agent_repo: AgentRepository,
        anti_fraud_service: Option<std::sync::Arc<AntiFraudService>>,
    ) -> Self {
        Self {
            player_repo,
            global_lb_repo,
            game_lb_service,
            agent_repo,
            anti_fraud_service,
        }
    }

    /// Handle player login (find or create).
    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: &str,
    ) -> Result<LoginResponse, AppError> {
        let wallet = request.wallet_address.trim().to_string();
        // ...
        tracing::info!(wallet = %wallet, "Player login attempt");

        if wallet.is_empty() {
            tracing::warn!("Login attempt with empty wallet address");
            return Err(AppError::BadRequest(
                "walletAddress is required".to_string(),
            ));
        }

        let name = request.name.unwrap_or_else(|| {
            let suffix = format!("{:x}", chrono::Utc::now().timestamp_millis())
                .chars()
                .rev()
                .take(8)
                .collect::<String>();
            format!("kult-player_{}", suffix)
        });

        let metadata: Option<Document> = request
            .metadata
            .and_then(|v| mongodb::bson::to_document(&v).ok());

        let (player, is_new) = self
            .player_repo
            .find_or_create(&wallet, &name, metadata)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, wallet = %wallet, "DB error during login");
                AppError::Internal(e)
            })?;

        if is_new {
            tracing::info!(wallet = %wallet, name = %name, "New player registered");

            // Process referral if one was provided
            if let Some(ref_code) = request.referral_code {
                if let Some(fraud_service) = &self.anti_fraud_service {
                    let player_id_str = player
                        .id
                        .map(|oid| oid.to_hex())
                        .unwrap_or_else(|| wallet.clone());

                    if let Err(e) = fraud_service
                        .process_referral_signup(&player_id_str, &ref_code, ip_address)
                        .await
                    {
                        tracing::error!(
                            error = %e,
                            wallet = %wallet,
                            "Failed to process referral signup"
                        );
                    } else {
                        tracing::info!(wallet = %wallet, code = %ref_code, "Referral pushed to validation queue");
                    }
                }
            }

            // Automatically generate a Web3 AI Agent identity for this new user
            if let Err(e) = self.agent_repo.create_agent_for_new_user(&wallet).await {
                tracing::error!(
                    error = %e,
                    wallet = %wallet,
                    "Failed to generate AI agent for new user during login registration"
                );
                // We don't necessarily want to block the user from logging in if agent gen fails,
                // but we must log it as a critical error.
            }
        } else {
            tracing::debug!(wallet = %wallet, "Existing player logged in");
        }

        let token = AuthService::sign_token(&player).map_err(|e| {
            tracing::error!(error = %e, "Failed to sign JWT token");
            AppError::Internal(e)
        })?;

        Ok(LoginResponse {
            token,
            player: PlayerInfo {
                id: player.id.map(|oid| oid.to_hex()).unwrap_or_default(),
                wallet_address: player.wallet_address,
                name: player.name,
            },
        })
    }

    /// Get a player's full profile with aggregated stats.
    pub async fn get_profile(
        &self,
        wallet_address: &str,
    ) -> Result<PlayerProfileResponse, AppError> {
        let wallet = wallet_address.trim().to_string();
        tracing::debug!(wallet = %wallet, "Fetching player profile");

        let player = self
            .player_repo
            .find_by_wallet(&wallet)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, wallet = %wallet, "DB error fetching player");
                AppError::Internal(e)
            })?
            .ok_or_else(|| {
                tracing::warn!(wallet = %wallet, "Player not found");
                AppError::NotFound("Player not found".to_string())
            })?;

        let global_entry = self
            .global_lb_repo
            .get_player_entry(&wallet)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch global leaderboard entry");
                AppError::Internal(e.to_string())
            })?;

        let (rank, total_score, level) = match global_entry {
            Some(entry) => {
                tracing::debug!(
                    rank = entry.rank,
                    level = entry.level,
                    "Player leaderboard entry found"
                );
                (Some(entry.rank), entry.score, entry.level)
            }
            None => {
                tracing::debug!(wallet = %wallet, "Player not ranked yet");
                (None, 0.0, 1)
            }
        };

        let game_scores = self
            .game_lb_service
            .fetch_scores_for_player(&wallet)
            .await
            .unwrap_or_default();

        tracing::debug!(games_played = game_scores.len(), "Game scores fetched");

        let game_scores_list: Vec<GameScoreEntry> = game_scores
            .into_iter()
            .map(|(id, score, weight, weighted, game_rank)| GameScoreEntry {
                identification: id,
                score,
                weight,
                weighted_score: weighted,
                rank: game_rank,
            })
            .collect();

        let profile = PlayerProfile {
            wallet_address: wallet,
            username: player.name,
            rank,
            total_score,
            level,
            total_games_played: game_scores_list.len() as u32,
            completed_quests: 0,
            game_scores_list,
            purchased_assets: extract_purchased_assets(player.metadata.as_ref()),
        };

        Ok(PlayerProfileResponse {
            cached: false,
            profile,
        })
    }

    /// Update a player's display name.
    pub async fn update_name(
        &self,
        wallet_address: &str,
        request: UpdateNameRequest,
    ) -> Result<UpdateNameResponse, AppError> {
        let new_name = request.name.trim();
        tracing::debug!(wallet = %wallet_address, new_name = %new_name, "Updating player name");

        if new_name.is_empty() {
            tracing::warn!(wallet = %wallet_address, "Attempted to set empty name");
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }

        if new_name.len() > 100 {
            tracing::warn!(wallet = %wallet_address, len = new_name.len(), "Name too long");
            return Err(AppError::BadRequest(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }

        let updated = self
            .player_repo
            .update_name(wallet_address, new_name)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "DB error updating player name");
                AppError::Internal(e)
            })?
            .ok_or_else(|| {
                tracing::warn!(wallet = %wallet_address, "Player not found for name update");
                AppError::NotFound("Player not found".to_string())
            })?;

        tracing::info!(wallet = %wallet_address, new_name = %updated.name, "Player name updated");
        Ok(UpdateNameResponse { name: updated.name })
    }
}

fn extract_purchased_assets(metadata: Option<&Document>) -> Option<Value> {
    let game_assets = metadata.and_then(|m| m.get("gameAssets"))?;
    serde_json::to_value(game_assets).ok()
}
