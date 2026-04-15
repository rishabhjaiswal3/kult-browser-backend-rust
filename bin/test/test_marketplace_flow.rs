/// Full marketplace integration test:
///
/// 1. Login to get auth token
/// 2. (Admin) Create listings (limited + unlimited)
/// 3. Browse listings (public)
/// 4. Get single listing detail
/// 5. Filter listings by gameIdentification / assetType
/// 6. Purchase a listing (create order)
/// 7. Get player's order history
/// 8. Get single order detail
/// 9. Purchase until sold out (limited listing)
/// 10. (Admin) Update a listing
/// 11. (Admin) Delist a listing
/// 12. (Admin) View all orders
/// 13. Validation edge cases
/// 14. Cleanup
///
/// Requires: cargo run (server on port 4000) + MongoDB running
/// Run: cargo run --features dev-bins --bin test_marketplace_flow
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

const API_URL: &str = "http://localhost:4000";
const TEST_WALLET: &str = "0xTEST_MARKETPLACE_FLOW_1234567890abcdef1234";

#[derive(Deserialize, Debug)]
struct ApiResponse<T> {
    ok: bool,
    data: T,
}

#[derive(Deserialize, Debug)]
struct ErrorBody {
    ok: bool,
    message: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ListingResponse {
    id: String,
    name: String,
    #[allow(dead_code)]
    asset_type: String,
    #[allow(dead_code)]
    game_identification: String,
    price: f64,
    supply: Option<u64>,
    remaining: Option<u64>,
    status: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ListingListResponse {
    listings: Vec<ListingResponse>,
    total: u64,
    #[allow(dead_code)]
    page: u32,
    #[allow(dead_code)]
    per_page: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct OrderResponse {
    id: String,
    #[allow(dead_code)]
    listing_id: String,
    #[allow(dead_code)]
    player_id: String,
    price_paid: f64,
    quantity: u32,
    status: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct OrderListResponse {
    orders: Vec<OrderResponse>,
    total: u64,
    #[allow(dead_code)]
    page: u32,
    #[allow(dead_code)]
    per_page: u32,
}

struct TestCtx {
    client: Client,
    token: String,
    listing_ids: Vec<String>,
    order_ids: Vec<String>,
}

impl TestCtx {
    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

// ── Helpers ──

/// Parse response as JSON, panicking with body text on failure.
async fn json_or_fail<T: serde::de::DeserializeOwned>(
    res: reqwest::Response,
    step: &str,
) -> T {
    let status = res.status();
    let body = res.text().await.unwrap_or_else(|e| format!("<read error: {}>", e));
    if !status.is_success() {
        println!("  ❌ {} — HTTP {} body: {}", step, status, body);
        std::process::exit(1);
    }
    serde_json::from_str(&body).unwrap_or_else(|e| {
        println!("  ❌ {} — parse error: {} body: {}", step, e, body);
        std::process::exit(1);
    })
}

fn pass(step: &str, detail: &str) {
    println!("  ✅ {} — {}", step, detail);
}

// ── Test Steps ──

async fn step_login(ctx: &mut TestCtx) {
    println!("\n━━━ Step 1: Login ━━━");
    let res = ctx
        .client
        .post(format!("{}/player/login", API_URL))
        .json(&json!({ "walletAddress": TEST_WALLET }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<LoginResponse> = json_or_fail(res, "Login").await;
    ctx.token = body.data.token;
    pass("Login", &format!("got token for {}", TEST_WALLET));
}

async fn step_admin_create_listings(ctx: &mut TestCtx) -> (String, String) {
    println!("\n━━━ Step 2: Admin — Create Listings ━━━");

    let test_game_slug = "test-game";

    // Limited listing (supply=3)
    let res = ctx
        .client
        .post(format!("{}/admin/marketplace", API_URL))
        .json(&json!({
            "name": "Test Golden Sword",
            "description": "Limited edition test weapon",
            "assetType": "weapon",
            "gameIdentification": test_game_slug,
            "price": 1.5,
            "supply": 3,
            "attributes": { "rarity": "legendary", "damage": 150 }
        }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingResponse> = json_or_fail(res, "Create limited listing").await;
    let limited_id = body.data.id.clone();
    ctx.listing_ids.push(limited_id.clone());
    assert_eq!(body.data.supply, Some(3));
    assert_eq!(body.data.remaining, Some(3));
    assert_eq!(body.data.status, "active");
    pass("Create limited listing", &format!("id={}, supply=3", limited_id));

    // Unlimited listing
    let res = ctx
        .client
        .post(format!("{}/admin/marketplace", API_URL))
        .json(&json!({
            "name": "Test Basic Skin",
            "description": "Always available test skin",
            "assetType": "skin",
            "gameIdentification": test_game_slug,
            "price": 0.5
        }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingResponse> = json_or_fail(res, "Create unlimited listing").await;
    let unlimited_id = body.data.id.clone();
    ctx.listing_ids.push(unlimited_id.clone());
    assert_eq!(body.data.supply, None);
    assert_eq!(body.data.remaining, None);
    pass("Create unlimited listing", &format!("id={}, unlimited", unlimited_id));

    (limited_id, unlimited_id)
}

async fn step_browse_listings(ctx: &TestCtx) {
    println!("\n━━━ Step 3: Browse Listings (public) ━━━");

    let res = ctx
        .client
        .get(format!("{}/marketplace", API_URL))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingListResponse> = json_or_fail(res, "Browse").await;
    assert!(body.data.total >= 2, "Expected at least 2 listings");
    pass("Browse", &format!("total={}, returned={}", body.data.total, body.data.listings.len()));
}

async fn step_get_single_listing(ctx: &TestCtx, listing_id: &str) {
    println!("\n━━━ Step 4: Get Single Listing ━━━");

    let res = ctx
        .client
        .get(format!("{}/marketplace/{}", API_URL, listing_id))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingResponse> = json_or_fail(res, "Get listing").await;
    assert_eq!(body.data.id, listing_id);
    pass("Get listing", &format!("name='{}', price={}", body.data.name, body.data.price));
}

async fn step_filter_listings(ctx: &TestCtx) {
    println!("\n━━━ Step 5: Filter Listings ━━━");

    let test_game_slug = "test-game";

    // By gameIdentification
    let res = ctx
        .client
        .get(format!("{}/marketplace?gameIdentification={}", API_URL, test_game_slug))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<ListingListResponse> = json_or_fail(res, "Filter gameIdentification").await;
    pass("Filter by gameIdentification", &format!("total={}", body.data.total));

    // By assetType
    let res = ctx
        .client
        .get(format!("{}/marketplace?assetType=weapon", API_URL))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<ListingListResponse> = json_or_fail(res, "Filter assetType").await;
    pass("Filter by assetType=weapon", &format!("total={}", body.data.total));

    // Pagination
    let res = ctx
        .client
        .get(format!("{}/marketplace?page=1&perPage=1", API_URL))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<ListingListResponse> = json_or_fail(res, "Pagination").await;
    assert!(body.data.listings.len() <= 1);
    pass("Pagination (perPage=1)", &format!("returned={}", body.data.listings.len()));
}

async fn step_purchase_listing(ctx: &mut TestCtx, listing_id: &str, quantity: u32) -> String {
    println!("\n━━━ Step 6: Purchase Listing ━━━");

    let res = ctx
        .client
        .post(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .json(&json!({
            "listingId": listing_id,
            "quantity": quantity
        }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<OrderResponse> = json_or_fail(res, "Purchase").await;
    let order_id = body.data.id.clone();
    ctx.order_ids.push(order_id.clone());
    assert_eq!(body.data.status, "completed");
    assert_eq!(body.data.quantity, quantity);
    pass(
        "Purchase",
        &format!("orderId={}, pricePaid={}, status={}", order_id, body.data.price_paid, body.data.status),
    );
    order_id
}

async fn step_get_order_history(ctx: &TestCtx) {
    println!("\n━━━ Step 7: Get Order History ━━━");

    let res = ctx
        .client
        .get(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .send()
        .await
        .unwrap();

    let body: ApiResponse<OrderListResponse> = json_or_fail(res, "Order history").await;
    assert!(body.data.total > 0, "Expected at least 1 order");
    pass("Order history", &format!("total={}", body.data.total));
}

async fn step_get_single_order(ctx: &TestCtx, order_id: &str) {
    println!("\n━━━ Step 8: Get Single Order ━━━");

    let res = ctx
        .client
        .get(format!("{}/marketplace/orders/{}", API_URL, order_id))
        .header("Authorization", ctx.auth())
        .send()
        .await
        .unwrap();

    let body: ApiResponse<OrderResponse> = json_or_fail(res, "Get order").await;
    assert_eq!(body.data.id, order_id);
    pass("Get order", &format!("id={}, status={}", body.data.id, body.data.status));
}

async fn step_purchase_until_sold_out(ctx: &mut TestCtx, limited_id: &str) {
    println!("\n━━━ Step 9: Purchase Until Sold Out ━━━");

    // supply=3, bought 1 in step 6. Buy 2 more.
    for i in 0..2 {
        let res = ctx
            .client
            .post(format!("{}/marketplace/orders", API_URL))
            .header("Authorization", ctx.auth())
            .json(&json!({ "listingId": limited_id, "quantity": 1 }))
            .send()
            .await
            .unwrap();

        let body: ApiResponse<OrderResponse> = json_or_fail(res, &format!("Purchase #{}", i + 2)).await;
        ctx.order_ids.push(body.data.id.clone());
        pass(&format!("Purchase #{}", i + 2), &format!("orderId={}", body.data.id));
    }

    // Verify sold_out
    let res = ctx
        .client
        .get(format!("{}/marketplace/{}", API_URL, limited_id))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingResponse> = json_or_fail(res, "Check sold_out").await;
    assert_eq!(body.data.status, "sold_out");
    assert_eq!(body.data.remaining, Some(0));
    pass("Sold out", &format!("status={}, remaining={:?}", body.data.status, body.data.remaining));

    // Try to buy again — must fail
    let res = ctx
        .client
        .post(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .json(&json!({ "listingId": limited_id, "quantity": 1 }))
        .send()
        .await
        .unwrap();

    assert!(!res.status().is_success(), "Expected rejection for sold_out listing");
    let err: ErrorBody = res.json().await.unwrap();
    pass("Buy sold_out rejected", &format!("message='{}'", err.message));
}

async fn step_admin_update_listing(ctx: &TestCtx, listing_id: &str) {
    println!("\n━━━ Step 10: Admin — Update Listing ━━━");

    let res = ctx
        .client
        .put(format!("{}/admin/marketplace/{}", API_URL, listing_id))
        .json(&json!({
            "name": "Updated Test Skin",
            "price": 0.75,
            "description": "Updated description"
        }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<ListingResponse> = json_or_fail(res, "Update listing").await;
    assert_eq!(body.data.name, "Updated Test Skin");
    assert_eq!(body.data.price, 0.75);
    pass("Update listing", &format!("name='{}', price={}", body.data.name, body.data.price));
}

async fn step_admin_delist(ctx: &TestCtx, listing_id: &str) {
    println!("\n━━━ Step 11: Admin — Delist ━━━");

    let res = ctx
        .client
        .delete(format!("{}/admin/marketplace/{}", API_URL, listing_id))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<Value> = json_or_fail(res, "Delist").await;
    pass("Delist", &format!("{}", body.data));

    // Verify gone from active feed
    let res = ctx
        .client
        .get(format!("{}/marketplace?assetType=skin", API_URL))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<ListingListResponse> = json_or_fail(res, "Delist verify").await;
    let found = body.data.listings.iter().any(|l| l.id == listing_id);
    assert!(!found, "Delisted listing still in active feed");
    pass("Delist verify", "gone from active feed");
}

async fn step_admin_get_all_orders(ctx: &TestCtx) {
    println!("\n━━━ Step 12: Admin — Get All Orders ━━━");

    let res = ctx
        .client
        .get(format!("{}/admin/marketplace/orders", API_URL))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<OrderListResponse> = json_or_fail(res, "Admin orders").await;
    assert!(body.data.total >= 3, "Expected >= 3 orders, got {}", body.data.total);
    pass("Admin get orders", &format!("total={}", body.data.total));
}

async fn step_validation_tests(ctx: &TestCtx) {
    println!("\n━━━ Step 13: Validation Edge Cases ━━━");

    // Invalid listing ID format
    let res = ctx.client
        .post(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .json(&json!({ "listingId": "not-valid", "quantity": 1 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("Invalid listingId", "rejected");

    // Zero quantity
    let res = ctx.client
        .post(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .json(&json!({ "listingId": "test-game", "quantity": 0 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("Zero quantity", "rejected");

    // Non-existent listing
    let res = ctx.client
        .post(format!("{}/marketplace/orders", API_URL))
        .header("Authorization", ctx.auth())
        .json(&json!({ "listingId": "bbbbbbbbbbbbbbbbbbbbbbbb", "quantity": 1 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("Non-existent listing", "rejected");

    // No auth on orders
    let res = ctx.client
        .post(format!("{}/marketplace/orders", API_URL))
        .json(&json!({ "listingId": "test-game", "quantity": 1 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("No auth", "rejected");

    // Negative price
    let res = ctx.client
        .post(format!("{}/admin/marketplace", API_URL))
        .json(&json!({ "name": "Bad", "assetType": "weapon", "gameIdentification": "test-game", "price": -1.0 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("Negative price", "rejected");

    // Empty name
    let res = ctx.client
        .post(format!("{}/admin/marketplace", API_URL))
        .json(&json!({ "name": "  ", "assetType": "weapon", "gameIdentification": "test-game", "price": 1.0 }))
        .send().await.unwrap();
    assert!(!res.status().is_success());
    pass("Empty name", "rejected");

    // 404 on non-existent listing
    let res = ctx.client
        .get(format!("{}/marketplace/bbbbbbbbbbbbbbbbbbbbbbbb", API_URL))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
    pass("Non-existent listing GET", "404");
}

async fn step_cleanup(ctx: &TestCtx) {
    println!("\n━━━ Cleanup ━━━");

    dotenvy::dotenv().ok();
    let _ = rustls::crypto::ring::default_provider().install_default();

    let db = kult_browser_backend_rust::mongo::connection::connect()
        .await
        .expect("Failed to connect to MongoDB for cleanup");

    let db_config = kult_browser_backend_rust::config::db_config::DbConfig::from_env();

    // Delete test listings
    let listings_coll = db.collection::<mongodb::bson::Document>(&db_config.marketplace_listings_collection);
    let mut deleted_listings = 0u64;
    for id in &ctx.listing_ids {
        if let Ok(oid) = mongodb::bson::oid::ObjectId::parse_str(id) {
            if let Ok(r) = listings_coll.delete_one(mongodb::bson::doc! { "_id": oid }).await {
                deleted_listings += r.deleted_count;
            }
        }
    }
    pass("Cleanup listings", &format!("deleted {}", deleted_listings));

    // Delete test orders
    let orders_coll = db.collection::<mongodb::bson::Document>(&db_config.marketplace_orders_collection);
    let mut deleted_orders = 0u64;
    for id in &ctx.order_ids {
        if let Ok(oid) = mongodb::bson::oid::ObjectId::parse_str(id) {
            if let Ok(r) = orders_coll.delete_one(mongodb::bson::doc! { "_id": oid }).await {
                deleted_orders += r.deleted_count;
            }
        }
    }
    pass("Cleanup orders", &format!("deleted {}", deleted_orders));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        MARKETPLACE FULL INTEGRATION TEST                ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let mut ctx = TestCtx {
        client: Client::new(),
        token: String::new(),
        listing_ids: Vec::new(),
        order_ids: Vec::new(),
    };

    step_login(&mut ctx).await;
    let (limited_id, unlimited_id) = step_admin_create_listings(&mut ctx).await;
    step_browse_listings(&ctx).await;
    step_get_single_listing(&ctx, &limited_id).await;
    step_filter_listings(&ctx).await;
    let order_id = step_purchase_listing(&mut ctx, &limited_id, 1).await;
    step_get_order_history(&ctx).await;
    step_get_single_order(&ctx, &order_id).await;
    step_purchase_until_sold_out(&mut ctx, &limited_id).await;
    step_admin_update_listing(&ctx, &unlimited_id).await;
    step_admin_delist(&ctx, &unlimited_id).await;
    step_admin_get_all_orders(&ctx).await;
    step_validation_tests(&ctx).await;
    step_cleanup(&ctx).await;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   🎉 ALL MARKETPLACE TESTS PASSED                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    Ok(())
}
