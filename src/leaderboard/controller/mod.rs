use crate::leaderboard::service::{GameLeaderboardService, GlobalLeaderboardService};

pub mod game_leaderboard_controller;
pub mod leaderboard_controller;

#[derive(Clone)]
pub struct LeaderboardState {
    pub game_service: GameLeaderboardService,
    pub global_service: GlobalLeaderboardService,
}
