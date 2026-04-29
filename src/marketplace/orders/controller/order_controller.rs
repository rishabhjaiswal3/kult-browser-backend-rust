// src/marketplace/orders/controller/order_controller.rs

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::handler::{ApiResponse, AppError};
use crate::marketplace::orders::dto::{
    ConfirmOrderRequest, CreateOrderRequest, OrdersQuery, PrepareOrderRequest,
};
use crate::marketplace::orders::service::OrderService;
use crate::middleware::AuthPlayer;

/// Shared state for marketplace order endpoints.
#[derive(Clone)]
pub struct OrdersState {
    pub order_service: OrderService,
}

/// POST /marketplace/orders
#[utoipa::path(
    post,
    path = "/api/marketplace/orders",
    summary = "Purchase a marketplace listing",
    description = "Creates an order for the authenticated player.",
    security(("bearer_auth" = [])),
    request_body = CreateOrderRequest,
    responses(
        (status = 200, description = "Order created", body = crate::openapi::OrderApiResponse),
        (status = 400, description = "Invalid request", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Listing not found", body = crate::openapi::ErrorResponse),
        (status = 409, description = "Listing sold out", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace Orders"
)]
pub async fn create_order(
    State(state): State<OrdersState>,
    auth: AuthPlayer,
    payload: Result<Json<CreateOrderRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .order_service
        .create_order(&auth.id, &auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /marketplace/orders/prepare
#[utoipa::path(
    post,
    path = "/api/marketplace/orders/prepare",
    summary = "Prepare a marketplace order",
    description = "Creates a pending order and returns canonical order details for payment.",
    security(("bearer_auth" = [])),
    request_body = PrepareOrderRequest,
    responses(
        (status = 200, description = "Order prepared", body = crate::openapi::PrepareOrderApiResponse),
        (status = 400, description = "Invalid request", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Listing not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace Orders"
)]
pub async fn prepare_order(
    State(state): State<OrdersState>,
    auth: AuthPlayer,
    payload: Result<Json<PrepareOrderRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .order_service
        .prepare_order(&auth.id, &auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /marketplace/orders/confirm
#[utoipa::path(
    post,
    path = "/api/marketplace/orders/confirm",
    summary = "Confirm a prepared marketplace order",
    description = "Confirms a pending order after payment transaction hash is available.",
    security(("bearer_auth" = [])),
    request_body = ConfirmOrderRequest,
    responses(
        (status = 200, description = "Order confirmed", body = crate::openapi::OrderApiResponse),
        (status = 400, description = "Invalid request", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Order not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace Orders"
)]
pub async fn confirm_order(
    State(state): State<OrdersState>,
    auth: AuthPlayer,
    payload: Result<Json<ConfirmOrderRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state.order_service.confirm_order(&auth.id, request).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /marketplace/orders
#[utoipa::path(
    get,
    path = "/api/marketplace/orders",
    summary = "Get player's order history",
    description = "Returns the authenticated player's marketplace orders.",
    security(("bearer_auth" = [])),
    params(OrdersQuery),
    responses(
        (status = 200, description = "Player's orders", body = crate::openapi::OrderListApiResponse),
        (status = 401, description = "Unauthorized", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace Orders"
)]
pub async fn get_orders(
    State(state): State<OrdersState>,
    auth: AuthPlayer,
    Query(query): Query<OrdersQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state
        .order_service
        .get_player_orders(&auth.id, page, per_page)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// GET /marketplace/orders/:id
#[utoipa::path(
    get,
    path = "/api/marketplace/orders/{id}",
    summary = "Get a single order",
    description = "Fetches a single order by ID (must belong to authenticated player).",
    security(("bearer_auth" = [])),
    params(
        ("id" = String, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Single order", body = crate::openapi::OrderApiResponse),
        (status = 401, description = "Unauthorized", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Order not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Marketplace Orders"
)]
pub async fn get_order(
    State(state): State<OrdersState>,
    auth: AuthPlayer,
    Path(id): Path<String>,
) -> Response {
    match state.order_service.get_order(&auth.id, &id).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}
