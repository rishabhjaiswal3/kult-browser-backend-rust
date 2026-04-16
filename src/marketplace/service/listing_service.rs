// src/marketplace/service/listing_service.rs

use mongodb::bson::{doc, oid::ObjectId};

use crate::handler::AppError;
use crate::marketplace::dto::{
    CreateListingRequest, DelistResponse, ListingListResponse, ListingResponse,
    UpdateListingRequest,
};
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

    // ── Public reads ──

    /// Get paginated active listings with optional filters.
    pub async fn get_listings(
        &self,
        game_identification: Option<String>,
        category: Option<String>,
        page: u32,
        per_page: u32,
    ) -> Result<ListingListResponse, AppError> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let limit = per_page as i64;

        let total = self
            .repo
            .count_active(game_identification.as_deref(), category.as_deref())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to count listings");
                AppError::Internal(format!("Failed to count listings: {}", e))
            })?;

        let listings = self
            .repo
            .find_active(
                game_identification.as_deref(),
                category.as_deref(),
                skip,
                limit,
            )
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
        let model = self.get_listing_model(id).await?;
        Ok(Self::to_response(model))
    }

    // ── Writes ──

    /// Validate, build model, insert, and return the created listing.
    pub async fn create_listing(
        &self,
        request: CreateListingRequest,
    ) -> Result<ListingModel, AppError> {
        let name = request.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest("name is required".to_string()));
        }
        if request.price < 0.0 {
            return Err(AppError::BadRequest(
                "price must be non-negative".to_string(),
            ));
        }
        let game_identification = request.game_identification.trim().to_string();
        if game_identification.is_empty() {
            return Err(AppError::BadRequest(
                "gameIdentification is required".to_string(),
            ));
        }
        let category = request.category.trim().to_string();
        if category.is_empty() {
            return Err(AppError::BadRequest("category is required".to_string()));
        }
        let currency = request.currency.trim().to_string();
        if currency.is_empty() {
            return Err(AppError::BadRequest("currency is required".to_string()));
        }

        let listing = ListingModel {
            id: None,
            name,
            short_description: request.short_description,
            long_description: request.long_description,
            asset_url: request.asset_url,
            price: request.price,
            category,
            currency,
            game_identification,
            status: "active".to_string(),
            created_at: None,
            updated_at: None,
        };

        let inserted_id = self
            .repo
            .create(listing)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create listing: {}", e)))?;

        self.repo
            .find_by_id(&inserted_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch listing: {}", e)))?
            .ok_or_else(|| AppError::Internal("Listing created but not found".to_string()))
    }

    /// Validate update fields, build BSON doc, apply update, return updated model.
    pub async fn update_listing(
        &self,
        id: &str,
        request: UpdateListingRequest,
    ) -> Result<ListingModel, AppError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid listing ID format".to_string()))?;

        let mut updates = mongodb::bson::Document::new();

        if let Some(name) = request.name {
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(AppError::BadRequest("name cannot be empty".to_string()));
            }
            updates.insert("name", name);
        }
        if let Some(sd) = request.short_description {
            updates.insert("shortDescription", sd);
        }
        if let Some(ld) = request.long_description {
            updates.insert("longDescription", ld);
        }
        if let Some(url) = request.asset_url {
            updates.insert("assetUrl", url);
        }
        if let Some(price) = request.price {
            if price < 0.0 {
                return Err(AppError::BadRequest(
                    "price must be non-negative".to_string(),
                ));
            }
            updates.insert("price", price);
        }
        if let Some(cat) = request.category {
            updates.insert("category", cat);
        }
        if let Some(cur) = request.currency {
            updates.insert("currency", cur);
        }

        if updates.is_empty() {
            return Err(AppError::BadRequest("No fields to update".to_string()));
        }

        self.repo
            .update(&oid, updates)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update listing: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))
    }

    /// Soft-delete: set status to delisted.
    pub async fn delist(&self, id: &str) -> Result<DelistResponse, AppError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid listing ID format".to_string()))?;

        self.repo
            .update(&oid, doc! { "status": "delisted" })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delist: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        Ok(DelistResponse {
            id: id.to_string(),
            message: "Listing delisted successfully".to_string(),
        })
    }

    // ── Internal helpers ──

    /// Fetch raw model by ID.
    pub async fn get_listing_model(&self, id: &str) -> Result<ListingModel, AppError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid listing ID format".to_string()))?;

        self.repo
            .find_by_id(&oid)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to fetch listing");
                AppError::Internal(format!("Failed to fetch listing: {}", e))
            })?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))
    }

    /// Map model → public DTO (no timestamps).
    pub fn to_response(listing: ListingModel) -> ListingResponse {
        ListingResponse {
            id: listing
                .id
                .map(|oid| oid.to_hex())
                .unwrap_or_default(),
            name: listing.name,
            short_description: listing.short_description,
            long_description: listing.long_description,
            asset_url: listing.asset_url,
            price: listing.price,
            category: listing.category,
            currency: listing.currency,
            game_identification: listing.game_identification,
            status: listing.status,
        }
    }
}
