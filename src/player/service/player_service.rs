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

/// Service layer for Player operations.
#[derive(Clone)]
pub struct PlayerService {
    player_repo: PlayerRepository,
    global_lb_repo: GlobalLeaderboardRepository,
    game_lb_service: GameLeaderboardService,
}

impl PlayerService {
    pub fn new(
        player_repo: PlayerRepository,
        global_lb_repo: GlobalLeaderboardRepository,
        game_lb_service: GameLeaderboardService,
    ) -> Self {
        Self {
            player_repo,
            global_lb_repo,
            game_lb_service,
        }
    }

    /// Handle player login (find or create).
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        let wallet = request.wallet_address.trim().to_lowercase();
        if wallet.is_empty() {
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

        let (player, _is_new) = self
            .player_repo
            .find_or_create(&wallet, &name, metadata)
            .await
            .map_err(|e| AppError::Internal(e))?;

        let token = AuthService::sign_token(&player).map_err(|e| AppError::Internal(e))?;

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
        let wallet = wallet_address.trim().to_lowercase();

        let player = self
            .player_repo
            .find_by_wallet(&wallet)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::NotFound("Player not found".to_string()))?;

        let global_entry = self
            .global_lb_repo
            .get_player_entry(&wallet)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let (rank, total_score, level) = match global_entry {
            Some(entry) => (Some(entry.rank), entry.score, entry.level),
            None => (None, 0.0, 1),
        };

        let game_scores = self
            .game_lb_service
            .fetch_scores_for_player(&wallet)
            .await
            .unwrap_or_default();

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

        if new_name.is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }

        if new_name.len() > 100 {
            return Err(AppError::BadRequest(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }

        let updated = self
            .player_repo
            .update_name(wallet_address, new_name)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::NotFound("Player not found".to_string()))?;

        Ok(UpdateNameResponse { name: updated.name })
    }
}
