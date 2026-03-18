use crate::content::repository::ContentConfigRepository;
use crate::content::service::ContentService;
use crate::game::repository::GameModelRepository;
use crate::handler::{ApiResponse, AppError};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

#[derive(Clone)]
pub struct ContentState {
    pub config_repo: Arc<ContentConfigRepository>,
    pub game_repo: Arc<GameModelRepository>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ContentParams {
    pub page: String,
    pub section: String,
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
}

/// GET /api/content
#[utoipa::path(
    get,
    path = "/api/content",
    params(ContentParams),
    responses(
        (status = 200, description = "Section content payload", body = crate::openapi::ContentApiResponse),
        (status = 400, description = "Invalid query parameters", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Configured content section not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Content"
)]
pub async fn get_content(
    State(state): State<ContentState>,
    query: Result<Query<ContentParams>, axum::extract::rejection::QueryRejection>,
) -> Response {
    // Handle query parsing errors with structured JSON
    let Query(params) = match query {
        Ok(q) => q,
        Err(rejection) => {
            return AppError::BadRequest(rejection.body_text()).into_response();
        }
    };

    let service = ContentService::new(&state.config_repo, &state.game_repo);
    let page_num = params.page_num.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    match service
        .get_content(&params.page, &params.section, page_num, page_size)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
