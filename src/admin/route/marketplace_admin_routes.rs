// src/admin/route/marketplace_admin_routes.rs

use crate::handler::{ApiResponse, AppError};
use crate::marketplace::dto::{CreateListingRequest, UpdateListingRequest};
use crate::marketplace::orders::dto::OrdersQuery;
use crate::marketplace::orders::service::OrderService;
use crate::marketplace::repository::ListingRepository;
use crate::marketplace::service::ListingService;
use crate::player::repository::PlayerRepository;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use mongodb::Database;

#[derive(Clone)]
struct AdminState {
    listing_service: ListingService,
    order_service: OrderService,
}

pub fn routes(db: Database) -> Router {
    let listing_repo = ListingRepository::new(&db);
    let order_repo = crate::marketplace::orders::repository::OrderRepository::new(&db);
    let player_repo = PlayerRepository::new(&db);
    let state = AdminState {
        listing_service: ListingService::new(listing_repo.clone()),
        order_service: OrderService::new(order_repo, listing_repo, player_repo),
    };

    Router::new()
        .route("/marketplace", post(create_listing))
        .route(
            "/marketplace/:id",
            put(update_listing).delete(delist_listing),
        )
        .route("/marketplace/orders", get(get_all_orders))
        .with_state(state)
}

async fn create_listing(
    State(state): State<AdminState>,
    payload: Result<Json<CreateListingRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state.listing_service.create_listing(request).await {
        Ok(model) => ApiResponse::success(ListingService::to_response(model)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn update_listing(
    State(state): State<AdminState>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateListingRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state.listing_service.update_listing(&id, request).await {
        Ok(model) => ApiResponse::success(ListingService::to_response(model)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn delist_listing(State(state): State<AdminState>, Path(id): Path<String>) -> Response {
    match state.listing_service.delist(&id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_all_orders(
    State(state): State<AdminState>,
    Query(query): Query<OrdersQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state.order_service.get_all_orders(page, per_page).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
