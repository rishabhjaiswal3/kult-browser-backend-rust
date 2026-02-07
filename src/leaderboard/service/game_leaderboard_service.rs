use crate::handler::AppError;
use crate::leaderboard::dto::{GameLeaderboardEntryDto, GameLeaderboardResponse};
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

    /// Fetch game leaderboard with pagination metadata.
    pub async fn fetch_leaderboard_paginated(
        &self,
        identification: &str,
        page: u32,
        page_size: u32,
    ) -> Result<GameLeaderboardResponse, AppError> {
        let skip = (page.saturating_sub(1)) * page_size;

        let config = self
            .config_repo
            .find_by_identification(identification)
            .await
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Leaderboard config not found for '{}'",
                    identification
                ))
            })?;

        let db = self.client.database(&config.db);
        let collection = db.collection::<Document>(&config.collection);

        let total_count = collection
            .count_documents(doc! {})
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count documents: {}", e)))?;

        let sort_order = config.order;

        let pipeline = vec![
            doc! {
                "$project": {
                    "player": format!("${}", config.person_key),
                    "score": format!("${}", config.score_key),
                }
            },
            doc! { "$sort": { "score": sort_order } },
            doc! { "$skip": skip as i64 },
            doc! { "$limit": page_size as i64 },
        ];

        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError::Internal(format!("Aggregation failed: {}", e)))?;

        let mut entries = Vec::new();
        let mut rank = skip + 1;

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
        {
            let player = doc.get_str("player").unwrap_or("unknown").to_string();
            let score = doc
                .get_f64("score")
                .or_else(|_| doc.get_i64("score").map(|s| s as f64))
                .or_else(|_| doc.get_i32("score").map(|s| s as f64))
                .unwrap_or(0.0);

            entries.push(GameLeaderboardEntryDto {
                rank,
                player,
                score,
                level: None,
                metadata: None,
            });
            rank += 1;
        }

        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64) / (page_size as f64)).ceil() as u32
        };

        Ok(GameLeaderboardResponse {
            entries,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    /// Legacy method for internal use (returns raw entries).
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

        let sort_order = config.order;

        let pipeline = vec![
            doc! {
                "$project": {
                    "player": format!("${}", config.person_key),
                    "score": format!("${}", config.score_key),
                }
            },
            doc! { "$sort": { "score": sort_order } },
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
            let player = doc.get_str("player").unwrap_or("unknown").to_string();
            let score = doc
                .get_f64("score")
                .or_else(|_| doc.get_i64("score").map(|s| s as f64))
                .or_else(|_| doc.get_i32("score").map(|s| s as f64))
                .unwrap_or(0.0);

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

    /// Fetch all game scores for a specific player.
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
            let db = self.client.database(&config.db);
            let coll = db.collection::<Document>(&config.collection);

            let pipeline = vec![
                doc! {
                    "$project": {
                        "person": format!("${}", config.person_key),
                        "score": format!("${}", config.score_key),
                    }
                },
                doc! { "$addFields": { "personLc": { "$toLower": "$person" } } },
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
