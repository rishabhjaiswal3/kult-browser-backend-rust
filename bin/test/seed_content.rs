use kult_browser_backend_rust::{
    content::{
        model::{ContentConfig, ContentType, FieldMapping},
        repository::ContentConfigRepository,
    },
    game::repository::GameModelRepository,
    mongo::connection,
};
use mongodb::bson::oid::ObjectId;

#[tokio::main]
async fn main() {
    println!("Connecting to database...");
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");

    let game_repo = GameModelRepository::new(&db);
    let config_repo = ContentConfigRepository::new(&db);

    println!("Fetching all games...");
    let games = game_repo
        .find_all(Some(100))
        .await
        .expect("Failed to fetch games");
    let game_ids: Vec<String> = games.into_iter().map(|g| g.identification).collect();

    if game_ids.is_empty() {
        eprintln!("No games found. Please run seed_games first.");
        return;
    }

    println!("Found {} games. Seeding Configs...", game_ids.len());

    let configs = vec![
        ContentConfig {
            id: ObjectId::new(),
            page: "home".to_string(),
            section: "top-picks".to_string(),
            content_type: ContentType::Game,
            content_order: game_ids.iter().take(4).cloned().collect(),
            field_mappings: Some(vec![
                FieldMapping {
                    response_key: "id".to_string(),
                    db_path: "identification".to_string(),
                },
                FieldMapping {
                    response_key: "title".to_string(),
                    db_path: "name.en".to_string(),
                },
                FieldMapping {
                    response_key: "cover_image".to_string(),
                    db_path: "images.hero.horizontal.en.url".to_string(),
                },
            ]),
        },
        ContentConfig {
            id: ObjectId::new(),
            page: "home".to_string(),
            section: "all-games".to_string(),
            content_type: ContentType::Game,
            content_order: game_ids.clone(),
            field_mappings: Some(vec![
                FieldMapping {
                    response_key: "id".to_string(),
                    db_path: "identification".to_string(),
                },
                FieldMapping {
                    response_key: "name".to_string(),
                    db_path: "name".to_string(), // Keep object
                },
                FieldMapping {
                    response_key: "image".to_string(),
                    db_path: "images.hero.horizontal.en.url".to_string(),
                },
                FieldMapping {
                    response_key: "slogan".to_string(),
                    db_path: "slogan.en".to_string(),
                },
            ]),
        },
    ];

    for mut config in configs {
        // Check if exists to preserve ID
        if let Ok(existing) = config_repo.find_config(&config.page, &config.section).await {
            config.id = existing.id;
        }

        match config_repo.upsert(&config).await {
            Ok(_) => println!("Upserted config for {}/{}", config.page, config.section),
            Err(e) => eprintln!("Failed to upsert config: {}", e),
        }
    }
}
