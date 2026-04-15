// src/marketplace/route/marketplace_routes.rs

use axum::{routing::get, Router};
use mongodb::Database;

use crate::marketplace::controller::{get_listing, get_listings, MarketplaceState};
use crate::marketplace::orders;
use crate::marketplace::repository::ListingRepository;
use crate::marketplace::service::ListingService;

/// Build routes for the marketplace module.
///
/// Routes:
/// - GET  /             - List active listings (paginated, filterable)
/// - GET  /:id          - Get single listing detail
/// - POST /orders       - Purchase a listing (auth required)
/// - GET  /orders       - Player's order history (auth required)
/// - GET  /orders/:id   - Single order detail (auth required)
pub fn routes(db: Database) -> Router {
    let listing_repo = ListingRepository::new(&db);
    let listing_service = ListingService::new(listing_repo);
    let state = MarketplaceState { listing_service };

    let orders_router = orders::route::routes(db);

    Router::new()
        .route("/", get(get_listings))
        .route("/:id", get(get_listing))
        .with_state(state)
        .nest("/orders", orders_router)
}
