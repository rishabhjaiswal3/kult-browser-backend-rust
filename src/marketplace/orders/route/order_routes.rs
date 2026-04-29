// src/marketplace/orders/route/order_routes.rs

use axum::{
    routing::{get, post},
    Router,
};
use mongodb::Database;

use crate::marketplace::orders::controller::{
    confirm_order, create_order, get_order, get_orders, prepare_order, OrdersState,
};
use crate::marketplace::orders::repository::OrderRepository;
use crate::marketplace::orders::service::OrderService;
use crate::marketplace::repository::ListingRepository;
use crate::player::repository::PlayerRepository;

/// Build routes for the marketplace orders submodule.
///
/// Routes (nested under /marketplace/orders):
/// - POST /       - Purchase a listing (auth required)
/// - GET  /       - Player's order history (auth required)
/// - GET  /:id    - Single order detail (auth required)
pub fn routes(db: Database) -> Router {
    let order_repo = OrderRepository::new(&db);
    let listing_repo = ListingRepository::new(&db);
    let player_repo = PlayerRepository::new(&db);
    let order_service = OrderService::new(order_repo, listing_repo, player_repo);
    let state = OrdersState { order_service };

    Router::new()
        .route("/", post(create_order).get(get_orders))
        .route("/prepare", post(prepare_order))
        .route("/confirm", post(confirm_order))
        .route("/:id", get(get_order))
        .with_state(state)
}
