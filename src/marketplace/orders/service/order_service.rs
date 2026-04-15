// src/marketplace/orders/service/order_service.rs

use mongodb::bson::oid::ObjectId;

use crate::handler::AppError;
use crate::marketplace::orders::dto::{CreateOrderRequest, OrderListResponse, OrderResponse};
use crate::marketplace::orders::model::OrderModel;
use crate::marketplace::orders::repository::OrderRepository;
use crate::marketplace::repository::ListingRepository;

#[derive(Clone)]
pub struct OrderService {
    order_repo: OrderRepository,
    listing_repo: ListingRepository,
}

impl OrderService {
    pub fn new(order_repo: OrderRepository, listing_repo: ListingRepository) -> Self {
        Self {
            order_repo,
            listing_repo,
        }
    }

    /// Purchase a listing — creates order and decrements supply.
    pub async fn create_order(
        &self,
        player_id: &str,
        request: CreateOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        tracing::debug!(player_id = %player_id, listing_id = %request.listing_id, "Creating order");

        let listing_oid = ObjectId::parse_str(&request.listing_id)
            .map_err(|_| AppError::BadRequest("Invalid listingId format".to_string()))?;

        if request.quantity == 0 {
            return Err(AppError::BadRequest("Quantity must be at least 1".to_string()));
        }

        // Fetch listing and validate
        let listing = self
            .listing_repo
            .find_by_id(&listing_oid)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch listing: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        if listing.status != "active" {
            return Err(AppError::BadRequest("Listing is not active".to_string()));
        }

        // Check supply
        if let Some(remaining) = listing.remaining {
            if remaining < request.quantity as u64 {
                return Err(AppError::BadRequest("Insufficient supply".to_string()));
            }
        }

        // Decrement supply atomically
        let updated_listing = self
            .listing_repo
            .decrement_supply(&listing_oid, request.quantity)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to decrement supply: {}", e)))?
            .ok_or_else(|| {
                AppError::Conflict("Listing sold out or no longer active".to_string())
            })?;

        // Create order
        let order = OrderModel {
            id: None,
            listing_id: listing_oid,
            player_id: player_id.to_string(),
            game_identification: listing.game_identification.clone(),
            price_paid: listing.price * request.quantity as f64,
            quantity: request.quantity,
            status: "completed".to_string(),
            tx_hash: request.tx_hash,
            created_at: None,
        };

        let order_id = self
            .order_repo
            .create(order.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create order: {}", e)))?;

        // Auto mark sold_out if remaining hit 0
        if updated_listing.remaining == Some(0) {
            let _ = self.listing_repo.mark_sold_out_if_empty(&listing_oid).await;
        }

        Ok(OrderResponse {
            id: order_id.to_hex(),
            listing_id: request.listing_id,
            player_id: player_id.to_string(),
            game_identification: listing.game_identification,
            price_paid: order.price_paid,
            quantity: order.quantity,
            status: "completed".to_string(),
            tx_hash: order.tx_hash,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Get player's order history.
    pub async fn get_player_orders(
        &self,
        player_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<OrderListResponse, AppError> {
        let skip = ((page.saturating_sub(1)) * per_page) as u64;
        let limit = per_page as i64;

        let total = self
            .order_repo
            .count_by_player(player_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count orders: {}", e)))?;

        let orders = self
            .order_repo
            .find_by_player(player_id, skip, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch orders: {}", e)))?;

        let items: Vec<OrderResponse> = orders.into_iter().map(Self::to_response).collect();

        Ok(OrderListResponse {
            orders: items,
            total,
            page,
            per_page,
        })
    }

    /// Get a single order by ID (player must own it).
    pub async fn get_order(
        &self,
        player_id: &str,
        order_id: &str,
    ) -> Result<OrderResponse, AppError> {
        let oid = ObjectId::parse_str(order_id)
            .map_err(|_| AppError::BadRequest("Invalid order ID format".to_string()))?;

        let order = self
            .order_repo
            .find_by_id(&oid)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch order: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

        if order.player_id != player_id {
            return Err(AppError::Forbidden(
                "You do not have access to this order".to_string(),
            ));
        }

        Ok(Self::to_response(order))
    }

    fn to_response(order: OrderModel) -> OrderResponse {
        OrderResponse {
            id: order.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            listing_id: order.listing_id.to_hex(),
            player_id: order.player_id,
            game_identification: order.game_identification,
            price_paid: order.price_paid,
            quantity: order.quantity,
            status: order.status,
            tx_hash: order.tx_hash,
            created_at: order
                .created_at
                .map(|dt| dt.try_to_rfc3339_string().unwrap_or_default())
                .unwrap_or_default(),
        }
    }
}
