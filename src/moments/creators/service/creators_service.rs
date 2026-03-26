use crate::handler::AppError;
use crate::moments::creators::dto::{
    CreatorLeaderboardEntry, CreatorLeaderboardResponse, CreatorProfileResponse,
};
use crate::moments::creators::model::CreatorAggregate;
use crate::moments::creators::repository::CreatorsRepository;
use crate::player::repository::PlayerRepository;

#[derive(Clone)]
pub struct CreatorsService {
    creators_repository: CreatorsRepository,
    player_repository: PlayerRepository,
}

impl CreatorsService {
    pub fn new(creators_repository: CreatorsRepository, player_repository: PlayerRepository) -> Self {
        Self {
            creators_repository,
            player_repository,
        }
    }

    pub async fn get_me(&self, wallet: &str) -> Result<CreatorProfileResponse, AppError> {
        let normalized_wallet = wallet.trim().to_lowercase();
        let aggregate = self
            .creators_repository
            .find_creator(&normalized_wallet)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Creator profile not found".to_string()))?;

        let rank = self
            .creators_repository
            .find_rank(&normalized_wallet)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("Creator profile not found".to_string()))?;

        let username = self
            .player_repository
            .find_by_wallet(&normalized_wallet)
            .await
            .map_err(AppError::Internal)?
            .map(|player| player.name);

        Ok(Self::to_profile_response(aggregate, username, rank))
    }

    pub async fn get_leaderboard(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<CreatorLeaderboardResponse, AppError> {
        let page = if page == 0 { 1 } else { page };
        let page_size = page_size.clamp(1, 50);

        let aggregates = self
            .creators_repository
            .list_leaderboard(page, page_size)
            .await
            .map_err(AppError::Internal)?;
        let total_count = self
            .creators_repository
            .count_creators()
            .await
            .map_err(AppError::Internal)?;

        let base_rank = ((page - 1) * page_size) as u64;
        let mut entries = Vec::with_capacity(aggregates.len());

        for (index, aggregate) in aggregates.into_iter().enumerate() {
            let username = self
                .player_repository
                .find_by_wallet(&aggregate.wallet_address)
                .await
                .map_err(AppError::Internal)?
                .map(|player| player.name);

            entries.push(Self::to_leaderboard_entry(
                aggregate,
                username,
                base_rank + index as u64 + 1,
            ));
        }

        let total_pages = if total_count == 0 {
            0
        } else {
            total_count.div_ceil(page_size as u64) as u32
        };

        Ok(CreatorLeaderboardResponse {
            entries,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    fn to_profile_response(
        aggregate: CreatorAggregate,
        username: Option<String>,
        rank: u64,
    ) -> CreatorProfileResponse {
        CreatorProfileResponse {
            wallet_address: aggregate.wallet_address,
            username,
            rank,
            total_moments: positive_u64(aggregate.total_moments),
            total_moment_likes: positive_u64(aggregate.total_moment_likes),
            total_moment_comments: positive_u64(aggregate.total_moment_comments),
            total_social_likes: positive_u64(aggregate.total_social_likes),
            validated_posts_count: positive_u64(aggregate.validated_posts_count),
            successful_referrals: positive_u64(aggregate.successful_referrals),
            total_score: positive_u64(aggregate.total_score),
        }
    }

    fn to_leaderboard_entry(
        aggregate: CreatorAggregate,
        username: Option<String>,
        rank: u64,
    ) -> CreatorLeaderboardEntry {
        CreatorLeaderboardEntry {
            rank,
            wallet_address: aggregate.wallet_address,
            username,
            total_moments: positive_u64(aggregate.total_moments),
            total_moment_likes: positive_u64(aggregate.total_moment_likes),
            total_moment_comments: positive_u64(aggregate.total_moment_comments),
            total_social_likes: positive_u64(aggregate.total_social_likes),
            validated_posts_count: positive_u64(aggregate.validated_posts_count),
            successful_referrals: positive_u64(aggregate.successful_referrals),
            total_score: positive_u64(aggregate.total_score),
        }
    }
}

fn positive_u64(value: i64) -> u64 {
    value.max(0) as u64
}
