// src/marketplace/service/listing_service.rs

use mongodb::bson::oid::ObjectId;

use crate::handler::AppError;
use crate::marketplace::dto::{ListingListResponse, ListingResponse};
use crate::marketplace::model::ListingModel;
use crate::marketplace::repository::ListingRepository;

#[derive(Clone)]
pub struct ListingService {
    repo: ListingRepository,
}

impl ListingService {
    pub fn new(repo: ListingRepository) -> Self {
        Self { repo }
    }

    /// Get paginated active listings with optional filters.
    pub async fn get_listings(
        &self,
        game_identification: Option<String>,
        asset_type: Option<String>,
        page: u32,
        per_page: u32,
    ) -> Result<ListingListResponse, AppError> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let limit = per_page as i64;

        let total = self
            .repo
            .count_active(game_identification.as_deref(), asset_type.as_deref())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to count listings");
                AppError::Internal(format!("Failed to count listings: {}", e))
            })?;

        let listings = self
            .repo
            .find_active(game_identification.as_deref(), asset_type.as_deref(), skip, limit)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch listings");
                AppError::Internal(format!("Failed to fetch listings: {}", e))
            })?;

        let items: Vec<ListingResponse> = listings.into_iter().map(Self::to_response).collect();

        Ok(ListingListResponse {
            listings: items,
            total,
            page,
            per_page,
        })
    }

    /// Get a single listing by ID.
    pub async fn get_listing(&self, id: &str) -> Result<ListingResponse, AppError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid listing ID format".to_string()))?;

        let listing = self
            .repo
            .find_by_id(&oid)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch listing");
                AppError::Internal(format!("Failed to fetch listing: {}", e))
            })?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        Ok(Self::to_response(listing))
    }

    pub(crate) fn to_response(listing: ListingModel) -> ListingResponse {
        ListingResponse {
            id: listing
                .id
                .map(|oid| oid.to_hex())
                .unwrap_or_default(),
            name: listing.name,
            description: listing.description,
            asset_type: listing.asset_type,
            game_identification: listing.game_identification,
            thumbnail_url: listing.thumbnail_url,
            price: listing.price,
            supply: listing.supply,
            remaining: listing.remaining,
            status: listing.status,
            attributes: listing
                .attributes
                .map(|d| mongodb::bson::from_document(d).unwrap_or_default()),
            created_at: listing
                .created_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                .unwrap_or_default(),
            updated_at: listing
                .updated_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                .unwrap_or_default(),
        }
    }
}
