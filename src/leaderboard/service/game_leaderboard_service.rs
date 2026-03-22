use crate::handler::AppError;
use crate::leaderboard::dto::{GameLeaderboardEntryDto, GameLeaderboardResponse};
use crate::leaderboard::model::{GameLeaderboardConfig, LeaderboardEntry};
use crate::leaderboard::repository::GameLeaderboardConfigRepository;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
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

        let pipeline = Self::build_leaderboard_pipeline(&config, skip, page_size);

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
            let entry = Self::entry_from_doc(doc, rank);
            entries.push(GameLeaderboardEntryDto::from_entry(entry, rank));
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

        let pipeline = Self::build_leaderboard_pipeline(&config, skip, limit);

        let mut cursor = collection
            .aggregate(pipeline)
            .await
            .map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        let mut rank = skip + 1;

        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            entries.push(Self::entry_from_doc(doc, rank));
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

    fn build_leaderboard_pipeline(
        config: &GameLeaderboardConfig,
        skip: u32,
        limit: u32,
    ) -> Vec<Document> {
        let mut project = doc! {
            "player": format!("${}", config.person_key),
            "score": format!("${}", config.score_key),
        };

        if let Some(metadata_projection) =
            Self::build_metadata_projection(config.projection.as_deref())
        {
            project.insert("metadata", Bson::Document(metadata_projection));
        }

        vec![
            doc! { "$project": project },
            doc! { "$sort": { "score": config.order } },
            doc! { "$skip": skip as i64 },
            doc! { "$limit": limit as i64 },
        ]
    }

    fn build_metadata_projection(projection: Option<&[String]>) -> Option<Document> {
        let mut metadata = Document::new();

        for path in projection.unwrap_or(&[]) {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('.').filter(|part| !part.is_empty()).collect();
            if parts.is_empty() {
                continue;
            }

            Self::insert_projection_path(&mut metadata, &parts, trimmed);
        }

        if metadata.is_empty() {
            None
        } else {
            Some(metadata)
        }
    }

    fn insert_projection_path(target: &mut Document, parts: &[&str], source_path: &str) {
        if parts.len() == 1 {
            target.insert(parts[0], format!("${source_path}"));
            return;
        }

        let key = parts[0].to_string();
        let needs_document = !matches!(target.get(&key), Some(Bson::Document(_)));
        if needs_document {
            target.insert(key.clone(), Bson::Document(Document::new()));
        }

        let nested = target
            .get_document_mut(&key)
            .expect("metadata projection branch should be a document");
        Self::insert_projection_path(nested, &parts[1..], source_path);
    }

    fn entry_from_doc(doc: Document, rank: u32) -> LeaderboardEntry {
        let player = doc.get_str("player").unwrap_or("unknown").to_string();
        let score = doc
            .get_f64("score")
            .or_else(|_| doc.get_i64("score").map(|value| value as f64))
            .or_else(|_| doc.get_i32("score").map(|value| value as f64))
            .unwrap_or(0.0);

        let metadata = doc
            .get_document("metadata")
            .ok()
            .and_then(|metadata| serde_json::to_value(metadata).ok())
            .filter(|metadata| {
                !matches!(metadata, serde_json::Value::Object(object) if object.is_empty())
            });

        LeaderboardEntry {
            rank,
            player,
            score,
            level: None,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameLeaderboardService;
    use mongodb::bson::doc;

    #[test]
    fn builds_nested_metadata_projection_from_config_paths() {
        let projection = vec![
            "username".to_string(),
            "profile.avatar.url".to_string(),
            " profile.country ".to_string(),
        ];

        let metadata = GameLeaderboardService::build_metadata_projection(Some(&projection))
            .expect("metadata projection should exist");

        assert_eq!(metadata.get_str("username").unwrap(), "$username");
        assert_eq!(
            metadata
                .get_document("profile")
                .unwrap()
                .get_document("avatar")
                .unwrap()
                .get_str("url")
                .unwrap(),
            "$profile.avatar.url"
        );
        assert_eq!(
            metadata
                .get_document("profile")
                .unwrap()
                .get_str("country")
                .unwrap(),
            "$profile.country"
        );
    }

    #[test]
    fn extracts_projected_metadata_from_aggregation_doc() {
        let entry = GameLeaderboardService::entry_from_doc(
            doc! {
                "player": "0xabc",
                "score": 42,
                "metadata": {
                    "username": "ankur",
                    "profile": {
                        "country": "IN"
                    }
                }
            },
            1,
        );

        assert_eq!(entry.player, "0xabc");
        assert_eq!(entry.score, 42.0);
        assert_eq!(entry.metadata.as_ref().unwrap()["username"], "ankur");
        assert_eq!(entry.metadata.as_ref().unwrap()["profile"]["country"], "IN");
    }
}
