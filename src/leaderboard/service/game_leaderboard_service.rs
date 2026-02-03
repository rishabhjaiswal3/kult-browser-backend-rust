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

    // Add this method to GameLeaderboardService impl block:

    /// Fetch all game scores for a specific player.
    ///
    /// Returns: Vec<(identification, score, weight, weighted_score, rank)>
    pub async fn fetch_scores_for_player(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<(String, f64, f64, f64, Option<u32>)>, String> {
        let configs = self
            .config_repo
            .find_all()
            .await
            .map_err(|e| format!("Failed to fetch configs: {}", e))?;
        let wallet = wallet_address.trim().to_lowercase();

        let mut results = Vec::new();

        for config in configs {
            // Build aggregation pipeline to get this player's score + rank
            let db = self.client.database(&config.db);
            let coll = db.collection::<Document>(&config.collection);

            let pipeline = vec![
                doc! {
                    "$project": {
                        "person": format!("${}", config.person_key),
                        "score": format!("${}", config.score_key),
                    }
                },
                doc! {
                    "$addFields": {
                        "personLc": { "$toLower": "$person" }
                    }
                },
                doc! {
                    "$setWindowFields": {
                        "sortBy": { "score": config.order },
                        "output": { "rank": { "$documentNumber": {} } }
                    }
                },
                doc! { "$match": { "personLc": &wallet } },
                doc! { "$limit": 1 },
            ];

            if let Ok(cursor) = coll.aggregate(pipeline).await {
                if let Ok(docs) = cursor.try_collect::<Vec<Document>>().await {
                    if let Some(doc) = docs.first() {
                        let score = doc.get_f64("score").unwrap_or(0.0);
                        let rank = doc.get_i32("rank").ok().map(|r| r as u32);
                        let weight = config.weight;
                        let weighted = score * weight;

                        results.push((
                            config.identification.clone(),
                            score,
                            weight,
                            weighted,
                            rank,
                        ));
                    }
                }
            }
        }

        Ok(results)
    }
}
