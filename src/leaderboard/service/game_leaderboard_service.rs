use crate::leaderboard::model::LeaderboardEntry;
use crate::leaderboard::repository::GameLeaderboardConfigRepository;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Client;

#[derive(Clone)]
pub struct GameLeaderboardService {
    config_repo: GameLeaderboardConfigRepository,
    client: Client,
}

impl GameLeaderboardService {
    pub fn new(config_repo: GameLeaderboardConfigRepository, client: Client) -> Self {
        Self {
            config_repo,
            client,
        }
    }

    pub async fn fetch_leaderboard(
        &self,
        identification: &str,
        skip: u32,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, String> {
        let config = self
            .config_repo
            .find_by_identification(identification)
            .await
            .ok_or_else(|| format!("Leaderboard config not found for {}", identification))?;

        let db = self.client.database(&config.db);
        let collection = db.collection::<Document>(&config.collection);

        // Build Pipeline
        let sort_order = config.order;

        // Dynamic Projection and Sorting
        // $project: { "player": "$walletAddress", "score": "$stats.total" } is not strictly needed
        // if we just fetch and map in Rust, but doing it in Mongo is cleaner for sorting if keys are deep.

        let pipeline = vec![
            // 1. Project standardized fields so we can sort/filter consistently
            doc! {
                "$project": {
                    "player": format!("${}", config.person_key),
                    "score": format!("${}", config.score_key),
                    // Preserve original document for metadata extraction if needed
                    // "original": "$$ROOT"
                }
            },
            // 2. Sort by standardized score
            doc! {
                "$sort": { "score": sort_order }
            },
            // 3. Pagination
            doc! { "$skip": skip as i64 },
            doc! { "$limit": limit as i64 },
        ];

        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        let mut rank = skip + 1;

        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            // Extract fields safely
            let player = doc.get_str("player").unwrap_or("unknown").to_string();

            // Score might be int or double
            let score = if let Ok(s) = doc.get_f64("score") {
                s
            } else if let Ok(s) = doc.get_i64("score") {
                s as f64
            } else if let Ok(s) = doc.get_i32("score") {
                s as f64
            } else {
                0.0
            };

            entries.push(LeaderboardEntry {
                rank,
                player,
                score,
                level: None,
                metadata: None,
            });
            rank += 1;
        }

        Ok(entries)
    }
}
