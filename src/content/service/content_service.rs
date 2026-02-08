use crate::content::model::{ContentResponse, FieldMapping};
use crate::content::repository::ContentConfigRepository;
use crate::game::repository::GameModelRepository;
use crate::handler::AppError;
use serde_json::Value;

pub struct ContentService<'a> {
    config_repo: &'a ContentConfigRepository,
    game_repo: &'a GameModelRepository,
}

impl<'a> ContentService<'a> {
    pub fn new(
        config_repo: &'a ContentConfigRepository,
        game_repo: &'a GameModelRepository,
    ) -> Self {
        Self {
            config_repo,
            game_repo,
        }
    }

    pub async fn get_content(
        &self,
        page: &str,
        section: &str,
        page_num: u32,
        page_size: u32,
    ) -> Result<ContentResponse, AppError> {
        tracing::debug!(page = %page, section = %section, page_num, page_size, "Fetching content");

        // 1. Fetch Config
        let config = self.config_repo.find_config(page, section).await?;
        tracing::debug!(
            config_items = config.content_order.len(),
            "Content config loaded"
        );

        let total_count = config.content_order.len() as u32;

        // 2. Pagination
        let start = ((page_num - 1) * page_size) as usize;
        if start >= config.content_order.len() {
            tracing::debug!("Pagination beyond available content, returning empty");
            return Ok(ContentResponse {
                content: vec![],
                total_content_count: total_count,
                page: page_num,
                page_size,
            });
        }

        let end = (start + page_size as usize).min(config.content_order.len());
        let target_ids = &config.content_order[start..end];
        tracing::debug!(target_count = target_ids.len(), "Fetching content items");

        // 3. Fetch Games
        let games_unordered = self
            .game_repo
            .find_by_ids(target_ids.to_vec())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch games from DB");
                AppError::Internal(format!("Failed to fetch games: {}", e))
            })?;

        tracing::debug!(fetched = games_unordered.len(), "Games fetched from DB");

        // Sort to match order
        let mut games_map: std::collections::HashMap<String, crate::game::model::GameModel> =
            games_unordered
                .into_iter()
                .map(|g| (g.identification.clone(), g))
                .collect();

        let mut ordered_content = Vec::new();
        for id in target_ids {
            if let Some(game) = games_map.remove(id) {
                if let Ok(game_val) = serde_json::to_value(game) {
                    let final_val = if let Some(ref mappings) = config.field_mappings {
                        apply_mapping(&game_val, mappings)
                    } else {
                        game_val
                    };
                    ordered_content.push(final_val);
                }
            }
        }

        tracing::debug!(
            returned = ordered_content.len(),
            total = total_count,
            "Content response prepared"
        );

        Ok(ContentResponse {
            content: ordered_content,
            total_content_count: total_count,
            page: page_num,
            page_size,
        })
    }
}

fn apply_mapping(source: &Value, mappings: &[FieldMapping]) -> Value {
    let mut mapped_obj = serde_json::Map::new();
    for mapping in mappings {
        let path_parts: Vec<&str> = mapping.db_path.split('.').collect();
        if let Some(val) = extract_value(source, &path_parts) {
            mapped_obj.insert(mapping.response_key.clone(), val.clone());
        }
    }
    Value::Object(mapped_obj)
}

fn extract_value<'b>(source: &'b Value, path: &[&str]) -> Option<&'b Value> {
    if path.is_empty() {
        return Some(source);
    }
    let key = path[0];
    let rest = &path[1..];
    source.get(key).and_then(|val| extract_value(val, rest))
}
