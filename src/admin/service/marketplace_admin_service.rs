// src/admin/service/marketplace_admin_service.rs

use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::handler::AppError;
use crate::marketplace::model::ListingModel;
use crate::marketplace::orders::dto::{OrderListResponse, OrderResponse};
use crate::marketplace::orders::repository::OrderRepository;
use crate::marketplace::repository::ListingRepository;
use crate::marketplace::service::ListingService;

#[derive(Clone)]
pub struct MarketplaceAdminService {
    listing_repo: ListingRepository,
    order_repo: OrderRepository,
}

/// Request payload for creating a marketplace listing (admin).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateListingRequest {
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub game_identification: String,
    pub thumbnail_url: Option<String>,
    pub price: f64,
    pub supply: Option<u64>,
    #[schema(value_type = Option<Object>)]
    pub attributes: Option<serde_json::Value>,
}

/// Request payload for updating a marketplace listing (admin).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListingRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub thumbnail_url: Option<String>,
    pub price: Option<f64>,
    pub supply: Option<u64>,
    #[schema(value_type = Option<Object>)]
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DelistResponse {
    pub id: String,
    pub message: String,
}

impl MarketplaceAdminService {
    pub fn new(listing_repo: ListingRepository, order_repo: OrderRepository) -> Self {
        Self {
            listing_repo,
            order_repo,
        }
    }

    /// Create a new marketplace listing.
    pub async fn create_listing(
        &self,
        request: CreateListingRequest,
    ) -> Result<crate::marketplace::dto::ListingResponse, AppError> {
        let name = request.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest("name is required".to_string()));
        }
        if request.price < 0.0 {
            return Err(AppError::BadRequest("price must be non-negative".to_string()));
        }

        let game_identification = request.game_identification.trim().to_string();
        if game_identification.is_empty() {
            return Err(AppError::BadRequest("gameIdentification is required".to_string()));
        }

        let attributes = request
            .attributes
            .map(|v| {
                mongodb::bson::to_document(&v)
                    .map_err(|e| AppError::BadRequest(format!("Invalid attributes: {}", e)))
            })
            .transpose()?;

        let listing = ListingModel {
            id: None,
            name,
            description: request.description,
            asset_type: request.asset_type,
            game_identification,
            thumbnail_url: request.thumbnail_url,
            price: request.price,
            supply: request.supply,
            remaining: request.supply, // remaining starts equal to supply
            status: "active".to_string(),
            attributes,
            created_at: None,
            updated_at: None,
        };

        let inserted_id = self
            .listing_repo
            .create(listing.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create listing: {}", e)))?;

        // Re-fetch to get timestamps
        let created = self
            .listing_repo
            .find_by_id(&inserted_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch listing: {}", e)))?
            .ok_or_else(|| AppError::Internal("Listing created but not found".to_string()))?;

        Ok(ListingService::to_response(created))
    }

    /// Update an existing listing.
    pub async fn update_listing(
        &self,
        id: &str,
        request: UpdateListingRequest,
    ) -> Result<crate::marketplace::dto::ListingResponse, AppError> {
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
        if let Some(desc) = request.description {
            updates.insert("description", desc);
        }
        if let Some(at) = request.asset_type {
            updates.insert("assetType", at);
        }
        if let Some(url) = request.thumbnail_url {
            updates.insert("thumbnailUrl", url);
        }
        if let Some(price) = request.price {
            if price < 0.0 {
                return Err(AppError::BadRequest("price must be non-negative".to_string()));
            }
            updates.insert("price", price);
        }
        if let Some(supply) = request.supply {
            updates.insert("supply", supply as i64);
            updates.insert("remaining", supply as i64);
        }
        if let Some(attrs) = request.attributes {
            let doc = mongodb::bson::to_document(&attrs)
                .map_err(|e| AppError::BadRequest(format!("Invalid attributes: {}", e)))?;
            updates.insert("attributes", doc);
        }

        if updates.is_empty() {
            return Err(AppError::BadRequest("No fields to update".to_string()));
        }

        let updated = self
            .listing_repo
            .update(&oid, updates)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update listing: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        Ok(ListingService::to_response(updated))
    }

    /// Soft-delete: set status to delisted.
    pub async fn delist(&self, id: &str) -> Result<DelistResponse, AppError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid listing ID format".to_string()))?;

        let updates = doc! { "status": "delisted" };
        let _ = self
            .listing_repo
            .update(&oid, updates)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delist: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        Ok(DelistResponse {
            id: id.to_string(),
            message: "Listing delisted successfully".to_string(),
        })
    }

    /// Get all orders (admin view, paginated).
    pub async fn get_all_orders(
        &self,
        page: u32,
        per_page: u32,
    ) -> Result<OrderListResponse, AppError> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let limit = per_page as i64;

        let total = self
            .order_repo
            .count_all()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count orders: {}", e)))?;

        let orders = self
            .order_repo
            .find_all(skip, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch orders: {}", e)))?;

        let items: Vec<OrderResponse> = orders
            .into_iter()
            .map(|o| OrderResponse {
                id: o.id.map(|oid| oid.to_hex()).unwrap_or_default(),
                listing_id: o.listing_id.to_hex(),
                player_id: o.player_id,
                game_identification: o.game_identification,
                price_paid: o.price_paid,
                quantity: o.quantity,
                status: o.status,
                tx_hash: o.tx_hash,
                created_at: o
                    .created_at
                    .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(OrderListResponse {
            orders: items,
            total,
            page,
            per_page,
        })
    }
}
