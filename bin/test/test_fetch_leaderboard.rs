use kult_browser_backend_rust::{
    leaderboard::{repository::GameLeaderboardConfigRepository, service::GameLeaderboardService},
    mongo::connection,
};

#[tokio::main]
async fn main() {
    println!("Connecting to database...");
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");

    // We need the client too, which is usually part of db/app state but here we can grab it from new connection or just use db.client() if exposed.
    // Wait, the connection::connect returns Database. Does it expose Client?
    // Let's assume we can get client from it or create a new client.
    // Actually, `kult_browser_backend_rust::mongo::connection::connect` returns `mongodb::Database`.
    // We cannot easily get Client from Database struct in mongodb driver v2/3 unless we use `db.client()`.
    // Let's check if `db.client()` is available. It is.

    let client = db.client();

    let config_repo = GameLeaderboardConfigRepository::new(&db);
    let service = GameLeaderboardService::new(config_repo, client.clone());

    let game_id = "zerogpool";
    println!("Fetching leaderboard for {}...", game_id);

    match service.fetch_leaderboard(game_id, 0, 5).await {
        Ok(entries) => {
            println!("Found {} entries:", entries.len());
            for entry in entries {
                println!("#{} - {}: {}", entry.rank, entry.player, entry.score);
            }
        }
        Err(e) => eprintln!("Error fetching leaderboard: {}", e),
    }

    let game_id_2 = "guesstheai";
    println!("\nFetching leaderboard for {}...", game_id_2);
    match service.fetch_leaderboard(game_id_2, 0, 5).await {
        Ok(entries) => {
            println!("Found {} entries:", entries.len());
            for entry in entries {
                println!("#{} - {}: {}", entry.rank, entry.player, entry.score);
            }
        }
        Err(e) => eprintln!("Error fetching leaderboard: {}", e),
    }
}
