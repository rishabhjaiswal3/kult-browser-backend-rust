use crate::game::dto::{
    AllGamesResponse, CategoriesResponse, GameDetailDto, GameDetailResponse, GameListItemDto,
};
use crate::game::model::GameModel;
use crate::game::repository::GameModelRepository;
use crate::handler::AppError;

/// Service layer for Game operations.
#[derive(Clone)]
pub struct GameService {
    repo: GameModelRepository,
}

impl GameService {
    pub fn new(repo: GameModelRepository) -> Self {
        Self { repo }
    }

    /// Get all games with optional search and pagination.
    pub async fn get_all_games(
        &self,
        search: Option<String>,
        page: u32,
        page_size: u32,
    ) -> Result<AllGamesResponse, AppError> {
        let skip = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let (games, total_count) = self
            .repo
            .find_all_paginated(search.as_deref(), skip, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch games: {}", e)))?;

        let game_dtos: Vec<GameListItemDto> = games.into_iter().map(Self::to_list_item).collect();

        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64) / (page_size as f64)).ceil() as u32
        };

        Ok(AllGamesResponse {
            games: game_dtos,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    /// Get a single game by identification.
    pub async fn get_game_by_identification(
        &self,
        identification: &str,
    ) -> Result<GameDetailResponse, AppError> {
        let game = self
            .repo
            .find_by_identification(identification)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch game: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Game '{}' not found", identification)))?;

        Ok(GameDetailResponse {
            game: Self::to_detail(game),
        })
    }

    /// Get all unique categories.
    pub async fn get_all_categories(&self) -> Result<CategoriesResponse, AppError> {
        let categories = self
            .repo
            .get_distinct_categories()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch categories: {}", e)))?;

        Ok(CategoriesResponse { categories })
    }

    fn to_list_item(game: GameModel) -> GameListItemDto {
        GameListItemDto {
            identification: game.identification,
            name: game.name,
            category: game.category,
            slogan: game.slogan,
            rating: game.rating,
            thumbnail: game.images.hero,
        }
    }

    fn to_detail(game: GameModel) -> GameDetailDto {
        GameDetailDto {
            identification: game.identification,
            name: game.name,
            url: game.url,
            category: game.category,
            about: game.about,
            rating: game.rating,
            thumbnail: game.images.hero,
        }
    }
}
