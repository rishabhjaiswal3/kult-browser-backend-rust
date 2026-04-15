use kult_browser_backend_rust::{
    leaderboard::{model::GameLeaderboardConfig, repository::GameLeaderboardConfigRepository},
    mongo::connection,
};
use mongodb::bson::oid::ObjectId;

#[tokio::main]
async fn main() {
    println!("Connecting to database...");
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");

    let repo = GameLeaderboardConfigRepository::new(&db);

    println!("Seeding Leaderboard Configs...");

    let configs = vec![
        GameLeaderboardConfig {
            id: Some(ObjectId::new()),
            identification: "guesstheai".to_string(),
            db: "guesstheai".to_string(),
            collection: "guesstheai_users".to_string(),
            score_key: "correctAnswers".to_string(),
            person_key: "walletAddress".to_string(),
            order: -1,
            weight: 1.0,
            projection: Some(vec![]),
        },
        GameLeaderboardConfig {
            id: Some(ObjectId::new()),
            identification: "zerodash".to_string(),
            db: "zerodash".to_string(),
            collection: "players".to_string(),
            score_key: "coins".to_string(),
            person_key: "walletAddress".to_string(),
            order: -1,
            weight: 1.0,
            projection: Some(vec![]),
        },
        GameLeaderboardConfig {
            id: Some(ObjectId::new()),
            identification: "zerogpool".to_string(),
            db: "zerogpool".to_string(),
            collection: "userdatas".to_string(),
            score_key: "stats.totalBallsPocketed".to_string(),
            person_key: "walletAddress".to_string(),
            order: -1,
            weight: 1.0,
            projection: Some(vec![]),
        },
    ];

    for mut config in configs {
        // Preserve ID if exists
        if let Some(existing) = repo.find_by_identification(&config.identification).await {
            config.id = existing.id;
        }

        match repo.upsert(&config).await {
            Ok(_) => println!("Upserted config for {}", config.identification),
            Err(e) => eprintln!("Failed to upsert {}: {}", config.identification, e),
        }
    }
}
