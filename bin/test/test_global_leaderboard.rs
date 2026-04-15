use kult_browser_backend_rust::{
    leaderboard::{
        repository::{GameLeaderboardConfigRepository, GlobalLeaderboardRepository},
        service::{GameLeaderboardService, GlobalLeaderboardService},
    },
    mongo::connection,
};

#[tokio::main]
async fn main() {
    println!("Connecting to database...");
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");
    let client = db.client();

    let config_repo = GameLeaderboardConfigRepository::new(&db);
    let global_repo = GlobalLeaderboardRepository::new(&db);
    let game_service = GameLeaderboardService::new(config_repo.clone(), client.clone());
    let global_service = GlobalLeaderboardService::new(config_repo, global_repo, game_service);

    println!("Refreshing Global Leaderboard (Aggregating scores)...");
    match global_service.refresh_global_leaderboard().await {
        Ok(count) => println!(
            "Successfully refreshed leaderboard! Total entries: {}",
            count
        ),
        Err(e) => eprintln!("Failed to refresh leaderboard: {}", e),
    }

    println!("\nFetching Global Leaderboard (Top 10)...");
    match global_service.get_global_leaderboard(0, 10).await {
        Ok(entries) => {
            if entries.is_empty() {
                println!("No entries found.");
            } else {
                println!(
                    "{:<5} | {:<42} | {:<6} | {:<5}",
                    "Rank", "Wallet", "Score", "Level"
                );
                println!("{}", "-".repeat(65));
                for entry in entries {
                    println!(
                        "{:<5} | {:<42} | {:<6} | {:<5}",
                        entry.rank, entry.wallet_address, entry.score, entry.level
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch leaderboard: {}", e),
    }
}
