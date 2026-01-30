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
}

impl GlobalLeaderboardService {
    pub fn new(
        config_repo: GameLeaderboardConfigRepository,
        global_repo: GlobalLeaderboardRepository,
        game_service: GameLeaderboardService,
    ) -> Self {
        Self {
            config_repo,
            global_repo,
            game_service,
        }
    }

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

    pub async fn refresh_global_leaderboard(&self) -> Result<usize, String> {
        // 1. Fetch all game configs
        let configs = self
            .config_repo
            .find_all()
            .await
            .map_err(|e| e.to_string())?;

        // 2. Fetch leaderboards concurrently
        let mut tasks = Vec::new();
        for config in &configs {
            // We fetch top 500 from each game to aggregate
            // This cloning of identification is needed for the async move block if we used one,
            // but here we call the service which is async.
            tasks.push(
                self.game_service
                    .fetch_leaderboard(&config.identification, 0, 500),
            );
        }

        let results = futures::future::join_all(tasks).await;

        // 3. Aggregate Scores
        let mut player_totals: HashMap<String, f64> = HashMap::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(entries) => {
                    let weight = configs[i].weight;
                    for entry in entries {
                        let total = player_totals.entry(entry.player).or_insert(0.0);
                        *total += entry.score * weight;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to fetch leaderboard for {}: {}",
                        configs[i].identification, e
                    );
                    // Continue even if one fails
                }
            }
        }

        // 4. Transform to Global Models using Industry Standard XP Curve
        // Logic: Quadratic Progression (Level = sqrt(Score))
        // This ensures levels are easy to get early on but get harder (standard RPG curve).
        // Edge Cases:
        // - Score 0 -> Level 1
        // - Score > 10,000 -> Cap at Level 100

        let mut global_entries = Vec::new();
        let now = Utc::now();

        // Sort by Score DESC logic (handled in step 5 generally, but let's do it here for rank assignment)
        let mut entries: Vec<(String, f64)> = player_totals.into_iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (i, (player, score)) in entries.into_iter().enumerate() {
            let rank = (i + 1) as u32;

            // Standard XP Curve: Level = sqrt(Score)
            // Example: 100 pts = Lvl 10. 10,000 pts = Lvl 100.
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

        // 5. Persist
        let count = global_entries.len();
        self.global_repo
            .replace_all(&global_entries)
            .await
            .map_err(|e| e.to_string())?;

        Ok(count)
    }
}
