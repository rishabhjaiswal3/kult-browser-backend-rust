use crate::game::repository::GameModelRepository;
use crate::handler::AppError;
use crate::leaderboard::dto::{GlobalLeaderboardEntryDto, GlobalLeaderboardResponse};
use crate::leaderboard::model::GlobalLeaderboardModel;
use crate::leaderboard::repository::{
    GameLeaderboardConfigRepository, GlobalLeaderboardRepository,
};
use crate::leaderboard::service::GameLeaderboardService;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct GlobalLeaderboardService {
    config_repo: GameLeaderboardConfigRepository,
    global_repo: GlobalLeaderboardRepository,
    game_service: GameLeaderboardService,
    game_repo: GameModelRepository,
}

impl GlobalLeaderboardService {
    pub fn new(
        config_repo: GameLeaderboardConfigRepository,
        global_repo: GlobalLeaderboardRepository,
        game_service: GameLeaderboardService,
        game_repo: GameModelRepository,
    ) -> Self {
        Self {
            config_repo,
            global_repo,
            game_service,
            game_repo,
        }
    }

    /// Get global leaderboard with pagination metadata.
    pub async fn get_global_leaderboard_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<GlobalLeaderboardResponse, AppError> {
        tracing::debug!(page, page_size, "Fetching global leaderboard");

        let skip = ((page.saturating_sub(1)) * page_size) as u64;
        let limit = page_size as i64;

        let entries = self
            .global_repo
            .get_global_ranking(skip, limit)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch global leaderboard from DB");
                AppError::Internal(format!("Failed to fetch leaderboard: {}", e))
            })?;

        let total_count = self.global_repo.count_all().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to count leaderboard entries");
            AppError::Internal(format!("Failed to count entries: {}", e))
        })?;

        tracing::debug!(
            fetched = entries.len(),
            total = total_count,
            "Leaderboard query completed"
        );

        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64) / (page_size as f64)).ceil() as u32
        };

        let entry_dtos: Vec<GlobalLeaderboardEntryDto> =
            entries.into_iter().map(|e| e.into()).collect();

        Ok(GlobalLeaderboardResponse {
            entries: entry_dtos,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    /// Legacy method for internal use (returns raw models).
    pub async fn get_global_leaderboard(
        &self,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<GlobalLeaderboardModel>, String> {
        self.global_repo
            .get_global_ranking(skip, limit)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn refresh_global_leaderboard(&self) -> Result<usize, AppError> {
        tracing::info!("Starting global leaderboard refresh");

        let configs = self.config_repo.find_all().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch leaderboard configs");
            AppError::Internal(format!("Failed to fetch configs: {}", e))
        })?;

        let config_ids: Vec<String> = configs
            .iter()
            .map(|config| config.identification.clone())
            .collect();

        let released_ids = self
            .game_repo
            .find_released_identifications(&config_ids)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch released games");
                AppError::Internal(format!("Failed to fetch released games: {}", e))
            })?;

        let mut released_configs = Vec::new();
        let mut skipped_ids = Vec::new();

        for config in configs {
            if released_ids.contains(&config.identification) {
                released_configs.push(config);
            } else {
                skipped_ids.push(config.identification.clone());
            }
        }

        tracing::debug!(
            game_count = released_configs.len(),
            skipped_unreleased = skipped_ids.len(),
            skipped_ids = ?skipped_ids,
            "Fetching scores from released games only"
        );

        let configs = released_configs;

        let mut tasks = Vec::new();
        for config in &configs {
            tasks.push(
                self.game_service
                    .fetch_leaderboard(&config.identification, 0, 500),
            );
        }

        let results = futures::future::join_all(tasks).await;

        let mut player_totals: HashMap<String, f64> = HashMap::new();
        let mut games_processed = 0;
        let mut games_failed = 0;

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(entries) => {
                    let weight = configs[i].weight;
                    tracing::debug!(
                        game = %configs[i].identification,
                        entries = entries.len(),
                        weight,
                        "Game leaderboard fetched"
                    );
                    for entry in entries {
                        let total = player_totals.entry(entry.player).or_insert(0.0);
                        *total += entry.score * weight;
                    }
                    games_processed += 1;
                }
                Err(e) => {
                    tracing::error!(
                        game = %configs[i].identification,
                        error = %e,
                        "Failed to fetch game leaderboard"
                    );
                    games_failed += 1;
                }
            }
        }

        tracing::debug!(
            games_processed,
            games_failed,
            unique_players = player_totals.len(),
            "Score aggregation completed"
        );

        let mut global_entries = Vec::new();
        let now = Utc::now();

        let mut entries: Vec<(String, f64)> = player_totals.into_iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (i, (player, score)) in entries.into_iter().enumerate() {
            let rank = (i + 1) as u32;
            let raw_level = score.sqrt().floor() as u32;
            let level = raw_level.clamp(1, 100);

            global_entries.push(GlobalLeaderboardModel {
                id: None,
                wallet_address: player,
                score,
                rank,
                level,
                updated_at: now,
                created_at: Some(now),
            });
        }

        let count = global_entries.len();
        self.global_repo
            .replace_all(&global_entries)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to persist global leaderboard");
                AppError::Internal(format!("Failed to persist leaderboard: {}", e))
            })?;

        tracing::info!(
            entries = count,
            games_processed,
            games_failed,
            "Global leaderboard refresh completed"
        );

        Ok(count)
    }
}
