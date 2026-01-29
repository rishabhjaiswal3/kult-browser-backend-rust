use crate::content::model::ContentResponse;
use crate::content::repository::ContentConfigRepository;
use crate::content::service::ContentService;
use crate::game::repository::GameModelRepository;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct ContentState {
    pub config_repo: Arc<ContentConfigRepository>,
    pub game_repo: Arc<GameModelRepository>,
}

#[derive(Debug, Deserialize)]
pub struct ContentParams {
    page: String,
    section: String,
    page_num: Option<u32>,
    page_size: Option<u32>,
    // lang: Option<String>, // Future
}

pub async fn get_content(
    State(state): State<ContentState>,
    Query(params): Query<ContentParams>,
) -> Result<Json<ContentResponse>, String> {
    let service = ContentService::new(&state.config_repo, &state.game_repo);
    let page_num = params.page_num.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    service
        .get_content(&params.page, &params.section, page_num, page_size)
        .await
        .map(Json)
}
