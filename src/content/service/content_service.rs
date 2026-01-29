use crate::content::model::ContentResponse;
use crate::content::repository::ContentConfigRepository;
use crate::game::repository::GameModelRepository;
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
    ) -> Result<ContentResponse, String> {
        // 1. Fetch Config
        let config = self
            .config_repo
            .find_config(page, section)
            .await
            .ok_or_else(|| "Section not found".to_string())?;

        let total_count = config.content_order.len() as u32;

        // 2. Pagination Logic (Slice the IDs)
        let start = ((page_num - 1) * page_size) as usize;
        if start >= config.content_order.len() {
            return Ok(ContentResponse {
                content: vec![],
                total_content_count: total_count,
                page: page_num,
                page_size,
            });
        }

        let end = (start + page_size as usize).min(config.content_order.len());
        let target_ids = &config.content_order[start..end]; // IDs to fetch

        // 3. Fetch Actual Content (Games only for now)
        let games_unordered = self
            .game_repo
            .find_by_ids(target_ids.to_vec())
            .await
            .unwrap_or_default();

        // Custom sort to match target_ids order
        let mut games_map: std::collections::HashMap<String, crate::game::model::GameModel> =
            games_unordered
                .into_iter()
                .map(|g| (g.identification.clone(), g))
                .collect();

        let mut ordered_content = Vec::new();
        for id in target_ids {
            if let Some(game) = games_map.remove(id) {
                // Convert to Value
                if let Ok(game_val) = serde_json::to_value(game) {
                    // Apply projection if attributes exist
                    let final_val = if let Some(ref attrs) = config.content_attributes {
                        apply_projection(game_val, attrs)
                    } else {
                        game_val
                    };
                    ordered_content.push(final_val);
                }
            }
        }

        Ok(ContentResponse {
            content: ordered_content,
            total_content_count: total_count,
            page: page_num,
            page_size,
        })
    }
}

fn apply_projection(source: Value, attributes: &[String]) -> Value {
    if attributes.is_empty() {
        return source;
    }
    // Deep projection could be complex. For now, we handle top-level keys.
    // If a key contains dots (e.g. "images.hero"), we imply fetching the top level "images".
    // A more advanced implementation would construct the nested object.

    if let Value::Object(map) = source {
        let mut projected = serde_json::Map::new();
        for attr in attributes {
            let top_key = attr.split('.').next().unwrap();
            if let Some(val) = map.get(top_key) {
                projected.insert(top_key.to_string(), val.clone());
            }
        }
        Value::Object(projected)
    } else {
        source // Cannot project non-object
    }
}
