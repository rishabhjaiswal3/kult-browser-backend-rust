// src/marketplace/orders/service/order_service.rs

use mongodb::bson::oid::ObjectId;
use nanoid::nanoid;
use std::env;

use crate::handler::AppError;
use crate::marketplace::orders::dto::{
    ConfirmOrderRequest, CreateOrderRequest, OrderListResponse, OrderResponse, PrepareOrderRequest,
    PrepareOrderResponse,
};
use crate::marketplace::orders::model::OrderModel;
use crate::marketplace::orders::repository::OrderRepository;
use crate::player::repository::PlayerRepository;
use crate::marketplace::repository::ListingRepository;
use crate::marketplace::orders::service::game_backend_sync::sync_external_game_entitlement;

#[derive(Clone)]
pub struct OrderService {
    order_repo: OrderRepository,
    listing_repo: ListingRepository,
    player_repo: PlayerRepository,
}

impl OrderService {
    pub fn new(
        order_repo: OrderRepository,
        listing_repo: ListingRepository,
        player_repo: PlayerRepository,
    ) -> Self {
        Self {
            order_repo,
            listing_repo,
            player_repo,
        }
    }

    /// Prepare an order before on-chain payment.
    pub async fn prepare_order(
        &self,
        player_id: &str,
        buyer_wallet: &str,
        request: PrepareOrderRequest,
    ) -> Result<PrepareOrderResponse, AppError> {
        let listing_oid = ObjectId::parse_str(&request.listing_id)
            .map_err(|_| AppError::BadRequest("Invalid listingId format".to_string()))?;

        if request.quantity == 0 {
            return Err(AppError::BadRequest("Quantity must be at least 1".to_string()));
        }
        let payment_token = request.payment_token.trim().to_string();
        if payment_token.is_empty() {
            return Err(AppError::BadRequest("paymentToken is required".to_string()));
        }

        let listing = self
            .listing_repo
            .find_by_id(&listing_oid)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch listing: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        if listing.status != "active" {
            return Err(AppError::BadRequest("Listing is not active".to_string()));
        }

        let order_id = format!("ord_{}", nanoid!(24));
        let order = OrderModel {
            id: None,
            listing_id: listing_oid,
            order_id: order_id.clone(),
            player_id: player_id.to_string(),
            buyer_wallet: buyer_wallet.to_string(),
            game_identification: listing.game_identification.clone(),
            payment_token: payment_token.clone(),
            price_paid: listing.price * request.quantity as f64,
            quantity: request.quantity,
            status: "pending".to_string(),
            tx_hash: None,
            created_at: None,
        };

        self.order_repo
            .create(order)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create pending order: {}", e)))?;

        let chain_id = env::var("MARKETPLACE_CHAIN_ID")
            .unwrap_or_else(|_| "8453".to_string())
            .parse::<u64>()
            .map_err(|_| AppError::Internal("MARKETPLACE_CHAIN_ID must be a valid u64".to_string()))?;
        let contract_address = env::var("MARKETPLACE_CONTRACT_ADDRESS")
            .or_else(|_| env::var("VITE_MARKETPLACE_CONTRACT_ADDRESS"))
            .unwrap_or_default();
        if contract_address.trim().is_empty() {
            return Err(AppError::Internal(
                "MARKETPLACE_CONTRACT_ADDRESS is not configured".to_string(),
            ));
        }

        Ok(PrepareOrderResponse {
            order_id,
            listing_id: request.listing_id,
            quantity: request.quantity,
            chain_id,
            contract_address,
            game_id: listing.game_identification,
            category: listing.category,
            item_id: listing
                .contract_item_id
                .unwrap_or_else(|| listing.id.map(|oid| oid.to_hex()).unwrap_or_default()),
            payment_token,
            expected_price: listing.price,
        })
    }

    /// Confirm a prepared order with transaction hash.
    pub async fn confirm_order(
        &self,
        player_id: &str,
        request: ConfirmOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        let tx_hash = request.tx_hash.trim();
        if tx_hash.is_empty() {
            return Err(AppError::BadRequest("txHash is required".to_string()));
        }

        let current = self
            .order_repo
            .find_by_order_id(&request.order_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch order: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

        if current.player_id != player_id {
            return Err(AppError::Forbidden(
                "You do not have access to this order".to_string(),
            ));
        }

        if current.status == "completed" {
            return Ok(Self::to_response(current));
        }

        let updated = self
            .order_repo
            .confirm_by_order_id(&request.order_id, tx_hash)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to confirm order: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

        self.apply_order_entitlement(&updated).await?;

        Ok(Self::to_response(updated))
    }

    /// Purchase a listing — creates an order.
    /// Legacy single-step API: immediately creates a completed order.
    pub async fn create_order(
        &self,
        player_id: &str,
        buyer_wallet: &str,
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

        // Create order
        let order = OrderModel {
            id: None,
            listing_id: listing_oid,
            order_id: format!("ord_{}", nanoid!(24)),
            player_id: player_id.to_string(),
            buyer_wallet: buyer_wallet.to_string(),
            game_identification: listing.game_identification.clone(),
            payment_token: String::new(),
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

        let mut saved_order = order.clone();
        saved_order.id = Some(order_id);
        self.apply_order_entitlement(&saved_order).await?;

        Ok(OrderResponse {
            id: order_id.to_hex(),
            order_id: order.order_id,
            listing_id: request.listing_id,
            player_id: player_id.to_string(),
            buyer_wallet: order.buyer_wallet,
            game_identification: listing.game_identification,
            payment_token: order.payment_token,
            price_paid: order.price_paid,
            quantity: order.quantity,
            status: "completed".to_string(),
            tx_hash: order.tx_hash,
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

        let items: Vec<OrderResponse> = orders.into_iter().map(Self::to_response).collect();

        Ok(OrderListResponse {
            orders: items,
            total,
            page,
            per_page,
        })
    }

    fn to_response(order: OrderModel) -> OrderResponse {
        OrderResponse {
            id: order.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            order_id: order.order_id,
            listing_id: order.listing_id.to_hex(),
            player_id: order.player_id,
            buyer_wallet: order.buyer_wallet,
            game_identification: order.game_identification,
            payment_token: order.payment_token,
            price_paid: order.price_paid,
            quantity: order.quantity,
            status: order.status,
            tx_hash: order.tx_hash,
        }
    }

    async fn apply_order_entitlement(&self, order: &OrderModel) -> Result<(), AppError> {
        let listing = self
            .listing_repo
            .find_by_id(&order.listing_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch listing: {}", e)))?
            .ok_or_else(|| AppError::NotFound("Listing not found".to_string()))?;

        let item_id = listing
            .contract_item_id
            .clone()
            .unwrap_or_else(|| order.listing_id.to_hex());

        self.player_repo
            .add_purchased_asset(
                &order.player_id,
                &listing.game_identification,
                &item_id,
                &listing.category,
                &listing.name,
                &order.order_id,
                order.tx_hash.as_deref(),
                order.quantity,
            )
            .await
            .map_err(AppError::Internal)?;

        // Optional fan-out: sync entitlement to game-specific backend if configured.
        if let Err(sync_err) = sync_external_game_entitlement(order, &listing, &item_id).await {
            tracing::warn!(
                order_id = %order.order_id,
                game = %listing.game_identification,
                error = %sync_err,
                "Failed to sync external game backend entitlement"
            );
        }

        Ok(())
    }
}
