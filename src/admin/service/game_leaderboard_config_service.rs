use crate::handler::AppError;
use crate::leaderboard::model::GameLeaderboardConfig;
use crate::leaderboard::repository::GameLeaderboardConfigRepository;
use serde::Serialize;

#[derive(Clone)]
pub struct GameLeaderboardConfigAdminService {
    repo: GameLeaderboardConfigRepository,
}

#[derive(Debug, Serialize)]
pub struct DeleteGameLeaderboardConfigResponse {
    pub identification: String,
    pub message: String,
}

impl GameLeaderboardConfigAdminService {
    pub fn new(repo: GameLeaderboardConfigRepository) -> Self {
        Self { repo }
    }

    pub async fn insert_config(
        &self,
        config: GameLeaderboardConfig,
    ) -> Result<GameLeaderboardConfig, AppError> {
        let mut config = Self::normalize_and_validate(config)?;

        let exists = self
            .repo
            .exists_by_identification(&config.identification)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to check leaderboard config: {}", e))
            })?;

        if exists {
            return Err(AppError::Conflict(format!(
                "Leaderboard config '{}' already exists",
                config.identification
            )));
        }

        let inserted_id = self.repo.insert(&config).await.map_err(|e| {
            AppError::Internal(format!("Failed to insert leaderboard config: {}", e))
        })?;

        config.id = Some(inserted_id);
        Ok(config)
    }

    pub async fn delete_config(
        &self,
        identification: &str,
    ) -> Result<DeleteGameLeaderboardConfigResponse, AppError> {
        let identification = identification.trim();
        if identification.is_empty() {
            return Err(AppError::BadRequest(
                "identification is required".to_string(),
            ));
        }

        let deleted = self
            .repo
            .delete_by_identification(identification)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to delete leaderboard config: {}", e))
            })?;

        if !deleted {
            return Err(AppError::NotFound(format!(
                "Leaderboard config '{}' not found",
                identification
            )));
        }

        Ok(DeleteGameLeaderboardConfigResponse {
            identification: identification.to_string(),
            message: format!("Deleted leaderboard config '{}'", identification),
        })
    }

    fn normalize_and_validate(
        mut config: GameLeaderboardConfig,
    ) -> Result<GameLeaderboardConfig, AppError> {
        config.id = None;
        config.identification = config.identification.trim().to_string();
        config.db = config.db.trim().to_string();
        config.collection = config.collection.trim().to_string();
        config.score_key = config.score_key.trim().to_string();
        config.person_key = config.person_key.trim().to_string();

        if config.identification.is_empty() {
            return Err(AppError::BadRequest(
                "identification is required".to_string(),
            ));
        }

        if config.db.is_empty() {
            return Err(AppError::BadRequest("db is required".to_string()));
        }

        if config.collection.is_empty() {
            return Err(AppError::BadRequest("collection is required".to_string()));
        }

        if config.score_key.is_empty() {
            return Err(AppError::BadRequest("scoreKey is required".to_string()));
        }

        if config.person_key.is_empty() {
            return Err(AppError::BadRequest("personKey is required".to_string()));
        }

        if !matches!(config.order, -1 | 1) {
            return Err(AppError::BadRequest("order must be 1 or -1".to_string()));
        }

        if !config.weight.is_finite() {
            return Err(AppError::BadRequest(
                "weight must be a finite number".to_string(),
            ));
        }

        config.projection = config.projection.take().and_then(|projection| {
            let projection: Vec<String> = projection
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect();

            if projection.is_empty() {
                None
            } else {
                Some(projection)
            }
        });

        Ok(config)
    }
}
